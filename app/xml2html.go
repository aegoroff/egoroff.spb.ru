package app

import (
	"encoding/xml"
	"log"
	"strings"
)

var tagsMap = map[string]string{
	"example": "pre",
	"quote":   "blockquote",
	"link":    "a",
	"center":  "div",
}

var decorations = map[string][]xml.Attr{
	"table":   {{Name: xml.Name{Local: "class"}, Value: "table table-condensed table-striped"}},
	"a":       {{Name: xml.Name{Local: "itemprop"}, Value: "url"}},
	"acronym": {{Name: xml.Name{Local: "class"}, Value: "initialism"}},
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
		if err != nil {
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
				start.Attr = append(start.Attr, attr...)
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
