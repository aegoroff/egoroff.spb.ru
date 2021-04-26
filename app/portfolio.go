package app

import (
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/framework"
	"egoroff.spb.ru/app/lib"
	"encoding/json"
	"fmt"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
	"os"
	"path"
	"sort"
	"strconv"
	"strings"
)

type Portfolio struct {
	documents    []*domain.Apache
	documentsMap map[string]*domain.Apache
	portfolio    map[int64]int64
}

func NewPortfolio(path string) *Portfolio {
	docs := readApacheDocs(path)
	docsMap := make(map[string]*domain.Apache)
	for _, doc := range docs {
		docsMap[doc.ID] = doc
	}

	portfolio := map[int64]int64{
		1:  19002,
		3:  24001,
		5:  23001,
		6:  21002,
		7:  22001,
		8:  21001,
		9:  13004,
		10: 6007,
		11: 18001,
		12: 12004,
		13: 12003,
		14: 1006,
		15: 6005,
		16: 16002,
		17: 12002,
		18: 6006,
		19: 17001,
		21: 11004,
		22: 9004,
		23: 13003,
		24: 16001,
		25: 9003,
		26: 14005,
		27: 5004,
		28: 11003,
		29: 15001,
		30: 8002,
	}

	return &Portfolio{
		documents:    docs,
		documentsMap: docsMap,
		portfolio:    portfolio,
	}
}

func (po *Portfolio) Documents() []*domain.Apache {
	return po.documents
}

func (po *Portfolio) Route(r *gin.Engine) {
	p := r.Group("/portfolio")
	{
		p.GET("/", po.index)
		p.GET("/:document", po.document)
	}
}

func (po *Portfolio) index(c *gin.Context) {
	ctx := framework.NewContext(c)
	appContext := ctx["ctx"].(*framework.Context)
	s := appContext.Section("portfolio")
	ctx["apache_docs"] = po.documents
	ctx["title"] = s.Title
	ctx["keywords"] = s.Keywords
	ctx["meta_description"] = s.Descr
	ctx["html_class"] = "portfolio"
	ctx["downloads"] = downloads()

	c.HTML(http.StatusOK, "portfolio/index.html", ctx)
}

func (po *Portfolio) document(c *gin.Context) {
	doc := lastParam(c, "document")

	if strings.HasSuffix(doc, ".html") {
		doc = doc[:len(doc)-len(".html")]
	}

	id, err := strconv.ParseInt(doc, 10, 64)
	if err == nil {
		newId, ok := po.portfolio[id]
		if ok {
			c.Redirect(http.StatusMovedPermanently, fmt.Sprintf("/blog/%d.html", newId))
		} else {
			framework.Error404(c)
		}
		return
	}

	d, ok := po.documentsMap[doc]

	if !ok {
		framework.Error404(c)
		return
	}

	docid, ok := po.documentsMap[doc]
	if !ok {
		framework.Error404(c)
		return
	}

	filer := lib.NewFiler(os.Stdout)
	b, err := filer.Read(path.Join("templates", "apache", docid.ID+".html"))
	if err != nil {
		log.Println(err)
	}

	ctx := framework.NewContext(c)
	ctx["content"] = string(b)
	ctx["title"] = d.Title
	ctx["keywords"] = d.Keywords

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

func readApacheDocs(path string) []*domain.Apache {
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
		var result = make([]*domain.Apache, 0, len(v))
		for k, val := range v {
			a := domain.Apache{
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

func downloads() []*domain.Folder {
	rep := db.NewRepository()
	folders := rep.Folders()
	for _, folder := range folders {
		for _, key := range folder.FileKeys {
			folder.Files = append(folder.Files, rep.File(key))
		}
	}
	return folders
}
