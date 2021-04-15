package app

import (
	"cloud.google.com/go/storage"
	"context"
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/framework"
	"egoroff.spb.ru/app/lib"
	"fmt"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
	"strings"
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

		blob := rep.Blob(file.BlobKey)
		if blob == nil {
			framework.Error404(c)
			return
		}

		ctx := context.Background()
		client, err := storage.NewClient(ctx)
		if err != nil {
			log.Println(err)
			framework.Error404(c)
			return
		}
		defer lib.Close(client)

		blobPath := strings.TrimPrefix(blob.GcsFileName, "/egoroff.appspot.com/")
		rc, err := client.Bucket("egoroff.appspot.com").Object(blobPath).NewReader(ctx)
		if err != nil {
			log.Println(err)
			framework.Error404(c)
			return
		}
		headers := make(map[string]string)
		headers["Content-Disposition"] = fmt.Sprintf("attachment; filename=%s", file.Filename)
		c.DataFromReader(http.StatusOK, file.Size, file.ContentType, rc, headers)
	})
}
