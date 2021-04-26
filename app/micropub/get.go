package micropub

import (
	"github.com/gin-gonic/gin"
	"net/http"
)

type SyndicateTo struct {
	UID  string `json:"uid"`
	Name string `json:"name"`
}

func getHandler(
	db getDB,
	mediaURL string,
	syndicateTo []SyndicateTo,
) gin.HandlerFunc {
	configHandler := configHandler(mediaURL, syndicateTo)
	sourceHandler := sourceHandler(db)
	syndicationHandler := syndicationHandler(syndicateTo)
	mediaEndpointHandler := mediaEndpointHandler(mediaURL)

	return func(c *gin.Context) {
		switch c.Query("q") {
		case "config":
			configHandler(c)
		case "media-endpoint":
			mediaEndpointHandler(c)
		case "source":
			sourceHandler(c)
		case "syndicate-to":
			syndicationHandler(c)
		}
	}
}

func sourceHandler(db getDB) gin.HandlerFunc {
	return func(c *gin.Context) {
		url := c.Query("url")
		properties := c.Request.Form["properties[]"]
		if len(properties) == 0 {
			property := c.Request.FormValue("properties")
			if len(property) > 0 {
				properties = []string{property}
			}
		}

		obj, err := db.Entry(url)
		if err != nil {
			c.AbortWithError(http.StatusNotFound, err)
			return
		}

		if len(properties) > 0 {
			for key := range obj {
				if !contains(key, properties) {
					delete(obj, key)
				}
			}
		}

		c.JSON(http.StatusOK, formToJSON(obj))
	}
}

func configHandler(mediaURL string, syndicateTo []SyndicateTo) gin.HandlerFunc {
	return func(c *gin.Context) {
		c.JSON(http.StatusOK, struct {
			Q             []string      `json:"q"`
			MediaEndpoint string        `json:"media-endpoint"`
			SyndicateTo   []SyndicateTo `json:"syndicate-to"`
		}{
			Q: []string{
				"config",
				"media-endpoint",
				"source",
				"syndicate-to",
			},
			MediaEndpoint: mediaURL,
			SyndicateTo:   syndicateTo,
		})
	}
}

func mediaEndpointHandler(mediaURL string) gin.HandlerFunc {
	return func(c *gin.Context) {
		c.JSON(http.StatusOK, struct {
			MediaEndpoint string `json:"media-endpoint"`
		}{
			MediaEndpoint: mediaURL,
		})
	}
}

func syndicationHandler(syndicateTo []SyndicateTo) gin.HandlerFunc {
	return func(c *gin.Context) {
		c.JSON(http.StatusOK, struct {
			SyndicateTo []SyndicateTo `json:"syndicate-to"`
		}{
			SyndicateTo: syndicateTo,
		})
	}
}

func contains(needle string, list []string) bool {
	for _, item := range list {
		if item == needle {
			return true
		}
	}

	return false
}
