package blog

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
		{"<div1><head>h</head><p>p</p></div1>", "<h2>h</h2><p>p</p>"},
		{"<div2><head>h</head><p>p</p></div2>", "<h3>h</h3><p>p</p>"},
		{"<div3><head>h</head><p>p</p></div3>", "<h4>h</h4><p>p</p>"},
		{"<div1><head>h</head><div2><head>h</head></div2></div1>", "<h2>h</h2><h3>h</h3>"},
		{"<p>s - s</p>", "<p>s&nbsp;&mdash; s</p>"},
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

func Test_parseHtml(t *testing.T) {
	var tests = []struct {
		raw    string
		parsed string
	}{
		{"<p>s - s</p>", "<p>s&nbsp;&mdash; s</p>"},
		{"<pre>s - s</pre>", "<pre>s - s</pre>"},
		{"<pre><p>s - s</p></pre>", "<pre><p>s - s</p></pre>"},
		{"<pre>&lt;html&gt;___&lt;/html&gt;</pre>", "<pre>&lt;html&gt;___&lt;/html&gt;</pre>"},
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
