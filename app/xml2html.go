package app

import (
	"encoding/xml"
	"io"
	"log"
	"strings"
)

var tagsMap = map[string]string{
	"example": "pre",
	"quote":   "blockquote",
	"link":    "a",
	"center":  "div",
}

type decorator func([]xml.Attr) []xml.Attr

var decorations = map[string]decorator{
	"table":   tableDecorator,
	"a":       linkDecorator,
	"acronym": acronymDecorator,
}

func tableDecorator(attr []xml.Attr) []xml.Attr {
	a := xml.Attr{Name: xml.Name{Local: "class"}, Value: "table table-condensed table-striped"}
	return append(attr, a)
}

func acronymDecorator(attr []xml.Attr) []xml.Attr {
	a := xml.Attr{Name: xml.Name{Local: "class"}, Value: "initialism"}
	return append(attr, a)
}

func linkDecorator(attr []xml.Attr) []xml.Attr {
	a := xml.Attr{Name: xml.Name{Local: "itemprop"}, Value: "url"}
	result := []xml.Attr{a}
	dir := ""
	file := ""
	for _, x := range attr {
		if x.Name.Local == "id" {
			if dir != "" {
				continue
			}
			switch x.Value {
			case "1", "53", "62":
				dir = "/portfolio/"
			case "2":
				dir = "/blog/"
			default:
				dir = "/"
			}
		} else if x.Name.Local == "name" {
			dir = "/blog/"
			file = x.Value + ".html"
		} else {
			result = append(result, x)
		}
	}
	if dir != "" {
		href := xml.Attr{Name: xml.Name{Local: "href"}, Value: dir + file}
		result = append(result, href)
	}
	return result
}

var decorationsIfNone = map[string][]xml.Attr{
	"example": {{Name: xml.Name{Local: "class"}, Value: "code"}},
	"center":  {{Name: xml.Name{Local: "style"}, Value: "text-align: center;"}},
}

func convert(x string) string {
	decoder := xml.NewDecoder(strings.NewReader(x))

	sb := strings.Builder{}
	encoder := xml.NewEncoder(&sb)

	encode := func(t xml.Token) {
		err := encoder.EncodeToken(t)
		if err != nil {
			log.Println(err)
		}
	}

	for {
		t, err := decoder.Token()
		if err != nil && err != io.EOF {
			log.Println(err)
		}
		if t == nil {
			break
		}

		switch se := t.(type) {
		case xml.StartElement:
			var start xml.StartElement
			if res, ok := tagsMap[se.Name.Local]; ok {
				start = xml.StartElement{
					Name: xml.Name{
						Local: res,
						Space: se.Name.Space,
					},
					Attr: se.Attr,
				}
				if attr, ok := decorationsIfNone[se.Name.Local]; ok {
					if len(start.Attr) == 0 {
						start.Attr = append(start.Attr, attr...)
					}
				}
			} else {
				start = se
			}

			if attr, ok := decorations[start.Name.Local]; ok {
				start.Attr = attr(start.Attr)
			}
			encode(start)
		case xml.CharData:
			encode(t)
		case xml.EndElement:
			if res, ok := tagsMap[se.Name.Local]; ok {
				converted := xml.EndElement{
					Name: xml.Name{
						Local: res,
						Space: se.Name.Space,
					},
				}
				encode(converted)
			} else {
				encode(t)
			}
		case xml.Directive:
			encode(t)
		}
	}
	err := encoder.Flush()
	if err != nil {
		log.Println(err)
	}
	return sb.String()
}
