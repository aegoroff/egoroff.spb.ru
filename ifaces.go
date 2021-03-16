package main

import "github.com/gin-gonic/gin"

type router interface {
	route(r *gin.Engine)
}
