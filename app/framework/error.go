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
		Text: "Требуется авторизация, а она или сломалась или ее у вас нет",
	})
}

func Error403(c *gin.Context) {
	errorer(c, "403", http.StatusForbidden, domain.Message{
		Type: "danger",
		Text: "Вам нельзя это использовать потому что недостаточно прав",
	})
}

func errorer(c *gin.Context, title string, code int, messages ...domain.Message) {
	ctx := NewContext(c, messages...)
	ctx["title"] = title
	c.HTML(code, "error.html", ctx)
}
