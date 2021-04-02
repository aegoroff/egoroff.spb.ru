package domain

import (
	"encoding/xml"
	"github.com/stretchr/testify/assert"
	"testing"
)

func Test_Urlset(t *testing.T) {
	// Arrange
	ass := assert.New(t)

	url1 := Url{
		Location:   "http://www.egoroff.spb.ru/",
		ChangeFreq: "weekly",
		Priority:   1.0,
	}

	url2 := Url{
		Location:   "http://www.egoroff.spb.ru/portfolio/",
		ChangeFreq: "weekly",
		Priority:   0.7,
	}

	urlset := UrlSet{
		XmlNS: "http://www.sitemaps.org/schemas/sitemap/0.9",
		Urls:  []Url{url1, url2},
	}

	// Act
	output, err := xml.MarshalIndent(urlset, "  ", "    ")

	// Assert
	ass.NoError(err)
	ass.Equal(`  <urlset xmlns="http://www.sitemaps.org/schemas/sitemap/0.9">
      <url>
          <loc>http://www.egoroff.spb.ru/</loc>
          <changefreq>weekly</changefreq>
          <priority>1</priority>
      </url>
      <url>
          <loc>http://www.egoroff.spb.ru/portfolio/</loc>
          <changefreq>weekly</changefreq>
          <priority>0.7</priority>
      </url>
  </urlset>`, string(output))
}
