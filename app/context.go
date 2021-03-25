package app

import (
	"encoding/json"
	"github.com/flosch/pongo2"
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"log"
	"os"
	"strings"
)

type Context struct {
	conf           *Config
	currentVersion string
	styles         []string
	graph          *Graph
}

func NewContext(gctx *gin.Context, messages ...Message) pongo2.Context {
	styles := []string{
		"min/style/style.min.css",
		//"min/style/adminstyle.min.css",
	}
	repo := NewRepository()
	config, err := repo.Config()
	if err != nil {
		log.Println(err)
	}

	root := readSiteMap()

	gr := NewGraph(root)
	ctx := &Context{
		currentVersion: os.Getenv("CURRENT_VERSION_ID"),
		styles:         styles,
		conf:           config,
		graph:          gr,
	}

	result := pongo2.Context{"ctx": ctx}

	result["sections"] = root.Children
	result["flashed_messages"] = messages
	session := sessions.Default(gctx)

	sub := session.Get("user-sub")
	if sub != nil {
		u, err := repo.UserByFederatedId(sub.(string))
		if err != nil {
			log.Println(err)
		} else {
			result["current_user"] = u
		}
	}

	if gctx.Request.RequestURI != "/" {
		bc, curr := breadcrumbs(gr, gctx.Request.RequestURI)
		result["breadcrumbs"] = bc
		result["current_section"] = curr
		result["title_path"] = revertPath(bc, config.BrandName)
	}

	return result
}

func revertPath(sections []*SiteSection, main string) string {
	var path []string
	for _, section := range sections {
		path = append(path, section.Title)
	}
	if len(path) == 0 {
		return ""
	}
	path = path[1:]
	for i, j := 0, len(path)-1; i < j; i, j = i+1, j-1 {
		path[i], path[j] = path[j], path[i]
	}
	path = append(path, main)
	return strings.Join(path, " | ")
}

func (c *Context) Section(id string) *SiteSection {
	return c.graph.Section(id)
}

func (c *Context) PathFor(id string) string {
	return c.graph.FullPath(id)
}

func breadcrumbs(gr *Graph, uri string) ([]*SiteSection, string) {
	parts := strings.Split(uri, "/")
	root := gr.g.Node(1).(*SiteSection)
	result := []*SiteSection{root}

	var current string
	for i, part := range parts {
		if part == "" {
			continue
		}
		s := gr.Section(part)
		if s != nil && gr.FullPath(s.Id) != uri {
			result = append(result, s)
		}
		if i == 1 {
			current = part
		}
	}
	return result, current
}

func readSiteMap() *SiteSection {
	fi := NewFiler(os.Stdout)
	b, err := fi.Read("static/map.json")
	if err != nil {
		log.Println(err)
	}

	var root SiteSection
	err = json.Unmarshal(b, &root)
	if err != nil {
		log.Println(err)
	}
	return &root
}
