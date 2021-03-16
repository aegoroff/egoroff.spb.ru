package main

import (
	"encoding/json"
	"fmt"
	"github.com/flosch/pongo2"
	"github.com/gin-gonic/gin"
	"github.com/jbowtie/gokogiri/xml"
	"github.com/jbowtie/ratago/xslt"
	"log"
	"net/http"
	"os"
	"sort"
	"strings"
)

type apacher struct {
	documents    []*Apache
	documentsMap map[string]*Apache
	styles       []string
}

func newApacher(path string, styles []string) *apacher {
	docs := readApacheDocs(path)
	docsMap := make(map[string]*Apache)
	for _, doc := range docs {
		docsMap[doc.ID] = doc
	}
	return &apacher{
		documents:    docs,
		documentsMap: docsMap,
		styles:       styles,
	}
}

func (a *apacher) route(r *gin.Engine) {
	r.GET("/portfolio/apache/:document.html", func(c *gin.Context) {
		doc := c.Param("document.html")
		doc = strings.TrimRight(doc, ".html")

		apache, ok := a.documentsMap[doc]

		if !ok {
			return
		}

		sheet := fmt.Sprintf("apache/%s.xsl", apache.Stylesheet)
		style, _ := xml.ReadFile(sheet, xml.StrictParseOption)
		stylesheet, _ := xslt.ParseStylesheet(style, sheet)

		//process the input
		input, _ := xml.ReadFile(fmt.Sprintf("apache/%s.xml", doc), xml.StrictParseOption)
		output, _ := stylesheet.Process(input, xslt.StylesheetOptions{false, nil})
		c.HTML(http.StatusOK, "apache.html", pongo2.Context{
			"content":            output,
			"current_version_id": os.Getenv("CURRENT_VERSION_ID"),
			"styles":             a.styles,
		})
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
