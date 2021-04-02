package framework

import (
	"egoroff.spb.ru/app/domain"
	"github.com/gin-gonic/gin"
	"net/http"
)

func Error404(c *gin.Context) {
	errorer(c, "404", http.StatusNotFound)
}

func Error401(c *gin.Context) {
	errorer(c, "401", http.StatusUnauthorized, domain.Message{
		Type: "danger",
		Text: "You are not authorized to view this content",
	})
}

func errorer(c *gin.Context, title string, code int, messages ...domain.Message) {
	ctx := NewContext(c, messages...)
	ctx["title"] = title
	c.HTML(code, "error.html", ctx)
}
