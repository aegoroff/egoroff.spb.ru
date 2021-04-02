package app

import "github.com/gin-gonic/gin"

type Router interface {
	Route(r *gin.Engine)
}
