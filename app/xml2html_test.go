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
		{"<p>a<example>b</example></p>", "<p>a<pre>b</pre></p>"},
		{"<p>a<br/></p>", "<p>a<br></br></p>"},
		{"<br/>", "<br></br>"},
		{"a", "a"},
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
