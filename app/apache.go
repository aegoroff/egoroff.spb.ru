package app

import (
	"encoding/json"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
	"os"
	"path"
	"sort"
	"strings"
)

type Apacher struct {
	documents    []*Apache
	documentsMap map[string]*Apache
}

func NewApacher(path string) *Apacher {
	docs := readApacheDocs(path)
	docsMap := make(map[string]*Apache)
	for _, doc := range docs {
		docsMap[doc.ID] = doc
	}
	return &Apacher{
		documents:    docs,
		documentsMap: docsMap,
	}
}

func (a *Apacher) Documents() []*Apache {
	return a.documents
}

func (a *Apacher) Route(r *gin.Engine) {
	r.GET("/portfolio/:document.html", func(c *gin.Context) {
		doc := c.Param("document.html")

		d, ok := a.documentsMap[strings.TrimRight(doc, ".html")]

		if !ok {
			ctx := NewContext("", c)
			ctx["title"] = "404"
			c.HTML(http.StatusNotFound, "error.html", ctx)
			return
		}

		filer := NewFiler(os.Stdout)
		b, err := filer.Read(path.Join("templates", "apache", doc))
		if err != nil {
			log.Println(err)
		}

		ctx := NewContext("", c)
		ctx["content"] = string(b)
		ctx["title"] = d.Title

		c.HTML(http.StatusOK, "apache.html", ctx)
	})
}

func readApacheDocs(path string) []*Apache {
	f, err := os.Open(path)
	if err != nil {
		log.Println(err)
	}
	dec := json.NewDecoder(f)
	for {
		var v map[string][]string
		if err := dec.Decode(&v); err != nil {
			log.Println(err)
			return nil
		}
		var result = make([]*Apache, 0, len(v))
		for k, val := range v {
			a := Apache{
				ID:          k,
				Stylesheet:  val[0],
				Title:       val[1],
				Description: val[2],
				Keywords:    val[3],
			}
			result = append(result, &a)
		}
		sort.Slice(result, func(i, j int) bool {
			return result[i].ID < result[j].ID
		})
		return result
	}
}
