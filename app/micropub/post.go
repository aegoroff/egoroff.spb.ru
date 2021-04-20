package micropub

import (
	"errors"
	"github.com/gin-gonic/gin"
	"io"
	"io/ioutil"
	"log"
	"mime"
	"mime/multipart"
	"net/http"
	"strings"
)

var malformedDeleteError = errors.New("could not decode json request: malformed delete")

type postDB interface {
	Create(data map[string][]interface{}) (string, error)
	Update(url string, replace, add, delete map[string][]interface{}, deleteAlls []string) error
	Delete(url string) error
	Undelete(url string) error
}

func postHandler(db postDB /*fw media.FileWriter*/) gin.HandlerFunc {
	h := micropubPostHandler{
		db: db,
		//fw: fw,
	}
	return func(c *gin.Context) {
		contentType := c.GetHeader("Content-Type")
		switch contentType {
		case "application/json":
			h.handleJSON(c)
		case "application/x-www-form-urlencoded":
			h.handleForm(c)
		case "multipart/form-data":
			h.handleMultiPart(c)
		}
	}
}

type micropubPostHandler struct {
	db postDB
	//fw media.FileWriter
}

func (h *micropubPostHandler) handleJSON(c *gin.Context) {
	v := jsonMicroformat{Properties: map[string][]interface{}{}}

	err := c.Bind(&v)
	if err != nil {
		log.Println(err)
		c.AbortWithError(http.StatusBadRequest, err)
		return
	}

	data := jsonToForm(v)

	if v.Action == "update" {
		replace := map[string][]interface{}{}
		for key, value := range v.Replace {
			if reservedKey(key) {
				continue
			}

			replace[key] = value
		}

		add := map[string][]interface{}{}
		for key, value := range v.Add {
			if reservedKey(key) {
				continue
			}

			add[key] = value
		}

		del := map[string][]interface{}{}
		var deleteAlls []string

		if ds, ok := v.Delete.([]interface{}); ok {
			for _, d := range ds {
				if dd, ok := d.(string); ok {
					deleteAlls = append(deleteAlls, dd)
				} else {
					c.AbortWithError(http.StatusBadRequest, malformedDeleteError)
					return
				}
			}
		} else if dm, ok := v.Delete.(map[string]interface{}); ok {
			for key, value := range dm {
				if reservedKey(key) {
					continue
				}

				if vs, ok := value.([]interface{}); ok {
					del[key] = vs
				} else {
					c.AbortWithError(http.StatusBadRequest, malformedDeleteError)
					return
				}
			}
		}

		if err := h.db.Update(v.URL, replace, add, del, deleteAlls); err != nil {
			c.AbortWithError(http.StatusInternalServerError, err)
			return
		}

		c.Status(http.StatusNoContent)
		return
	}

	if v.Action == "delete" {
		h.delete(c, v.URL)
		return
	}

	if v.Action == "undelete" {
		h.undelete(c, v.URL)
		return
	}

	h.create(c, data)
}

func (h *micropubPostHandler) handleForm(c *gin.Context) {
	var req interface{}
	err := c.Bind(&req)
	if err != nil {
		c.AbortWithError(http.StatusBadRequest, err)
		return
	}

	data := map[string][]interface{}{}
	for key, values := range c.Request.Form {
		if reservedKey(key) {
			continue
		}

		if strings.HasSuffix(key, "[]") {
			key := key[:len(key)-2]
			for _, value := range values {
				if value != "" {
					data[key] = append(data[key], value)
				}
			}
		} else {
			if values[0] != "" {
				data[key] = []interface{}{values[0]}
			}
		}
	}

	if c.Request.FormValue("action") == "delete" {
		h.delete(c, c.Request.FormValue("url"))
		return
	}

	if c.Request.FormValue("action") == "undelete" {
		h.undelete(c, c.Request.FormValue("url"))
		return
	}

	h.create(c, data)
}

func (h *micropubPostHandler) handleMultiPart(c *gin.Context) {
	_, params, err := mime.ParseMediaType(c.GetHeader("Content-Type"))
	if err != nil {
		log.Println("ERR micropub-parse-multi-part;", err)
		return
	}

	data := map[string][]interface{}{}
	parts := multipart.NewReader(c.Request.Body, params["boundary"])

	for {
		p, err := parts.NextPart()
		if err == io.EOF {
			break
		}
		if err != nil {
			log.Println("ERR micropub-read-multi-part;", err)
			c.AbortWithError(http.StatusInternalServerError, err)
			return
		}

		mt, _, er := mime.ParseMediaType(p.Header.Get("Content-Disposition"))
		if er != nil || mt != "form-data" {
			continue
		}

		key := p.FormName()

		switch key {
		case "photo", "video", "audio":
			//location, err := h.fw.WriteFile(ps["filename"], p.Header.Get("Content-Type"), p)
			//if err != nil {
			//	log.Println("ERR micropub-photo;", err)
			//	continue
			//}

			//data[key] = []interface{}{location}

		case "photo[]", "video[]", "audio[]":
			//location, err := h.fw.WriteFile(ps["filename"], p.Header.Get("Content-Type"), p)
			//if err != nil {
			//	log.Println("ERR micropub-photo;", err)
			//	continue
			//}
			//
			//key := key[:len(key)-2]
			//data[key] = append(data[key], location)

		default:
			if reservedKey(key) {
				continue
			}

			slurp, err := ioutil.ReadAll(p)
			if err != nil {
				log.Fatal(err)
			}

			value := string(slurp)
			if value == "" {
				continue
			}

			if strings.HasSuffix(key, "[]") {
				key := key[:len(key)-2]
				data[key] = append(data[key], value)
			} else {
				data[key] = []interface{}{value}
			}
		}
	}

	h.create(c, data)
}

func (h *micropubPostHandler) create(c *gin.Context, data map[string][]interface{}) {
	//if clientID := auth.ClientID(r); clientID != "" {
	//	data["hx-client-id"] = []interface{}{clientID}
	//}

	location, err := h.db.Create(data)
	if err != nil {
		c.AbortWithError(http.StatusInternalServerError, err)
		return
	}

	c.Header("Location", location)
	c.Status(http.StatusCreated)
}

func (h *micropubPostHandler) delete(c *gin.Context, url string) {
	if err := h.db.Delete(url); err != nil {
		c.AbortWithError(http.StatusInternalServerError, err)
		return
	}

	c.Status(http.StatusNoContent)
}

func (h *micropubPostHandler) undelete(c *gin.Context, url string) {
	if err := h.db.Undelete(url); err != nil {
		c.AbortWithError(http.StatusInternalServerError, err)
		return
	}

	c.Status(http.StatusNoContent)
}

func reservedKey(key string) bool {
	return key == "access_token" || key == "action" || key == "url"
}
