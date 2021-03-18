package app

import (
	"github.com/flosch/pongo2"
	"log"
	"os"
)

type Context struct {
	conf           *Config
	currentVersion string
	styles         []string
	htmlClass      string
}

func NewContext(htmlClass string) pongo2.Context {
	styles := []string{
		"min/style/style.min.css",
		//"min/style/adminstyle.min.css",
	}
	repo := NewRepository()
	c, err := repo.Config()
	if err != nil {
		log.Println(err)
	}
	ctx := &Context{
		htmlClass:      htmlClass,
		currentVersion: os.Getenv("CURRENT_VERSION_ID"),
		styles:         styles,
		conf:           c,
	}

	return pongo2.Context{
		"ctx": ctx,
	}
}
