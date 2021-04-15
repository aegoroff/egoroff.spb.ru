package app

import (
	"cloud.google.com/go/storage"
	"context"
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/framework"
	"egoroff.spb.ru/app/lib"
	"github.com/gin-gonic/gin"
	"io/ioutil"
	"log"
	"net/http"
)

type download struct {
}

func NewDownload() Router {
	return &download{}
}

func (s *download) Route(r *gin.Engine) {
	r.GET("/f/:key", func(c *gin.Context) {
		rep := db.NewRepository()
		key := c.Param("key")
		file := rep.FileByUid(key)
		if file == nil {
			framework.Error404(c)
			return
		}

		ctx := context.Background()
		client, err := storage.NewClient(ctx)
		if err != nil {
			return
		}
		defer lib.Close(client)
		rc, err := client.Bucket("egoroff.appspot.com").Object(file.BlobKey).NewReader(ctx)
		if err != nil {
			log.Println(err)
			framework.Error404(c)
			return
		}
		data, err := ioutil.ReadAll(rc)
		if err != nil {
			log.Println(err)
			framework.Error404(c)
			return
		}
		c.Data(http.StatusOK, file.ContentType, data)
	})
}
