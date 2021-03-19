package app

import (
	"encoding/json"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
	"os"
	"path"
	"sort"
)

type Portfolio struct {
	documents    []*Apache
	documentsMap map[string]*Apache
}

func NewPortfolio(path string) *Portfolio {
	docs := readApacheDocs(path)
	docsMap := make(map[string]*Apache)
	for _, doc := range docs {
		docsMap[doc.ID] = doc
	}
	return &Portfolio{
		documents:    docs,
		documentsMap: docsMap,
	}
}

func (po *Portfolio) Documents() []*Apache {
	return po.documents
}

func (po *Portfolio) Route(r *gin.Engine) {
	p := r.Group("/portfolio")
	{
		p.GET("/", func(c *gin.Context) {
			ctx := NewContext("portfolio", c)
			appContext := ctx["ctx"].(*Context)
			s := appContext.Section("portfolio")
			ctx["apache_docs"] = po.documents
			ctx["title"] = s.Title

			c.HTML(http.StatusOK, "portfolio/index.html", ctx)
		})

		p.GET(":document.html", func(c *gin.Context) {
			doc := c.Param("document.html")

			k := doc[:len(doc)-len(".html")]

			d, ok := po.documentsMap[k]

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

			c.HTML(http.StatusOK, "portfolio/apache.html", ctx)
		})
	}
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
