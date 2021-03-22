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
		p.GET("/", po.index)
		r.GET("/apache", po.index)

		p.GET(":document", po.document)
		p.GET("/:document/:document", po.document)
	}
}

func (po *Portfolio) index(c *gin.Context) {
	ctx := NewContext(c)
	appContext := ctx["ctx"].(*Context)
	s := appContext.Section("portfolio")
	ctx["apache_docs"] = po.documents
	ctx["title"] = s.Title
	ctx["html_class"] = "portfolio"

	c.HTML(http.StatusOK, "portfolio/index.html", ctx)
}

func (po *Portfolio) document(c *gin.Context) {
	doc := lastParam(c, "document")

	if strings.HasSuffix(doc, ".html") {
		doc = doc[:len(doc)-len(".html")]
	}

	d, ok := po.documentsMap[doc]

	if !ok {
		error404(c)
		return
	}

	filer := NewFiler(os.Stdout)
	b, err := filer.Read(path.Join("templates", "apache", doc+".html"))
	if err != nil {
		log.Println(err)
	}

	ctx := NewContext(c)
	ctx["content"] = string(b)
	ctx["title"] = d.Title

	c.HTML(http.StatusOK, "portfolio/apache.html", ctx)
}

func lastParam(c *gin.Context, name string) string {
	var params []string
	for _, param := range c.Params {
		if param.Key == name {
			params = append(params, param.Value)
		}
	}
	if params == nil || len(params) == 0 {
		return ""
	}
	return params[len(params)-1]
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
