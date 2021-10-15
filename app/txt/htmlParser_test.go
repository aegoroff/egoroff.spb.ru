package txt

import (
	"github.com/stretchr/testify/assert"
	"testing"
)

func Test_parseHtml(t *testing.T) {
	var tests = []struct {
		raw    string
		parsed string
	}{
		{"<p>s- s</p>", "<p>s&nbsp;&mdash; s</p>"},
		{"<p>s - s</p>", "<p>s&nbsp;&mdash; s</p>"},
		{"<p>s -&nbsp;s</p>", "<p>s&nbsp;&mdash;\u00a0s</p>"},
		{"<pre>s - s</pre>", "<pre>s - s</pre>"},
		{"<pre><p>s - s</p></pre>", "<pre><p>s - s</p></pre>"},
		{"<pre>&lt;html&gt;___&lt;/html&gt;</pre>", "<pre>&lt;html&gt;___&lt;/html&gt;</pre>"},
		{"<p>123-456</p>", "<p>123&minus;456</p>"},
	}

	for _, test := range tests {
		t.Run(test.raw, func(t *testing.T) {
			// Arrange
			ass := assert.New(t)

			// Act
			html := ParseHtml(test.raw)

			// Assert
			ass.Equal(test.parsed, html)
		})
	}
}
