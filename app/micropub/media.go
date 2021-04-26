package micropub

import (
	"errors"
	"github.com/gin-gonic/gin"
	"io"
	"log"
	"mime"
	"mime/multipart"
	"net/http"
	"sync"
)

func mediaHandler(fw fileWriter) gin.HandlerFunc {
	mh := newMHandler(fw)
	return func(c *gin.Context) {
		switch c.Request.Method {
		case http.MethodGet:
			mh.get(c)
		case http.MethodPost:
			mh.post(c)
		case http.MethodOptions:
			c.Header("Accept", "GET,POST")
		default:
			c.Header("Accept", "GET,POST")
			c.AbortWithStatusJSON(http.StatusMethodNotAllowed, gin.H{})
		}
	}
}

type mHandler struct {
	fw fileWriter

	mu      sync.RWMutex
	lastURL string
}

func newMHandler(fw fileWriter) *mHandler {
	return &mHandler{fw: fw}
}

func (h *mHandler) get(c *gin.Context) {
	if c.Request.FormValue("q") != "last" {
		c.AbortWithStatusJSON(http.StatusBadRequest, gin.H{})
		return
	}

	h.mu.RLock()
	lastURL := h.lastURL
	h.mu.RUnlock()

	c.JSON(http.StatusOK, gin.H{"url": lastURL})
}

func (h *mHandler) post(c *gin.Context) {
	//if !h.hasScope(w, r, "media", "create") {
	//	return
	//}

	mediaType, params, err := mime.ParseMediaType(c.GetHeader("Content-Type"))
	if err != nil {
		log.Println("ERR media-upload;", err)
		return
	}
	if mediaType != "multipart/form-data" {
		log.Println("ERR media-upload; bad mediaType")
		c.AbortWithError(http.StatusUnsupportedMediaType, errors.New("expected content-type of multipart/form-data"))
		return
	}

	parts := multipart.NewReader(c.Request.Body, params["boundary"])

	part, err := parts.NextPart()
	if err == io.EOF {
		log.Println("ERR media-upload; empty form")
		c.AbortWithError(http.StatusBadRequest, errors.New("expected multipart form to contain a part"))
		return
	}
	if err != nil {
		log.Println("ERR media-upload;", err)
		c.AbortWithError(http.StatusBadRequest, errors.New("problem reading multipart form"))
		return
	}

	mt, ps, er := mime.ParseMediaType(part.Header.Get("Content-Disposition"))
	if er != nil || mt != "form-data" || ps["name"] != "file" {
		log.Println("ERR media-upload; expected only single part")
		c.AbortWithError(http.StatusBadRequest, errors.New("request must only contain a part named 'file'"))
		return
	}

	location, err := h.fw.WriteFile(ps["filename"], part.Header.Get("Content-Type"), part)
	if err != nil {
		log.Println("ERR media-upload;", err)
		c.AbortWithError(http.StatusInternalServerError, errors.New("problem writing media to file"))
		return
	}

	h.mu.Lock()
	h.lastURL = location
	h.mu.Unlock()

	c.Header("Location", location)
	c.Status(http.StatusCreated)
}
