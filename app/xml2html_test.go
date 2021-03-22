package app

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func Test_convert(t *testing.T) {
	var tests = []struct {
		xml  string
		html string
	}{
		{"<example x=\"y\">a</example><p>b</p>", "<pre x=\"y\">a</pre><p>b</p>"},
		{"<p>a<example>b</example></p>", "<p>a<pre class=\"code\">b</pre></p>"},
		{"<p>a<br/></p>", "<p>a<br></br></p>"},
		{"<br/>", "<br></br>"},
		{"a", "a"},
		{"<link id=\"1\">a</link>", "<a itemprop=\"url\" href=\"/portfolio/\">a</a>"},
		{"<link id=\"53\">a</link>", "<a itemprop=\"url\" href=\"/portfolio/\">a</a>"},
		{"<link id=\"62\">a</link>", "<a itemprop=\"url\" href=\"/portfolio/\">a</a>"},
		{"<link id=\"2\">a</link>", "<a itemprop=\"url\" href=\"/blog/\">a</a>"},
		{"<link id=\"3\">a</link>", "<a itemprop=\"url\" href=\"/\">a</a>"},
		{"<link id=\"2\" name=\"10\">a</link>", "<a itemprop=\"url\" href=\"/blog/10.html\">a</a>"},
		{"<link id=\"1\" name=\"10\">a</link>", "<a itemprop=\"url\" href=\"/blog/10.html\">a</a>"},
		{"<link name=\"10\" id=\"1\">a</link>", "<a itemprop=\"url\" href=\"/blog/10.html\">a</a>"},
	}

	for _, test := range tests {
		t.Run(test.xml, func(t *testing.T) {
			// Arrange
			ass := assert.New(t)

			// Act
			html := convert(test.xml)

			// Assert
			ass.Equal(test.html, html)
		})
	}
}
