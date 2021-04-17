package blog

import (
	"encoding/xml"
	"fmt"
	"io"
	"log"
	"regexp"
	"strings"
)

var tagsMap = map[string]string{
	"example": "pre",
	"quote":   "blockquote",
	"link":    "a",
	"center":  "div",
	"div1":    "2",
	"div2":    "3",
	"div3":    "4",
	"head":    "h",
}

var parentsMap = map[string]string{
	"2": "2",
	"3": "3",
	"4": "4",
}

var sym = map[string]string{
	"1/2": "frac12",
	"1/4": "frac14",
	"3/4": "frac34",
}

func entity(s string) string {
	symbol := s
	if r, ok := sym[s]; ok {
		symbol = r
	}
	return fmt.Sprintf("&%s;", symbol)
}

type decorator func([]xml.Attr) []xml.Attr

var decorations = map[string]decorator{
	"table":   tableDecorator,
	"a":       linkDecorator,
	"acronym": acronymDecorator,
}

var skips = map[string]struct{}{
	"pre":     {},
	"a":       {},
	"script":  {},
	"code":    {},
	"example": {},
	"link":    {},
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

	parent := ""
	current := ""

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
			current = se.Name.Local
			if res, ok := tagsMap[se.Name.Local]; ok {
				if _, ok := parentsMap[res]; ok {
					parent = res
					continue
				}
				if res == "h" {
					res += parent
				}
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
			if _, ok := skips[current]; ok {
				encode(t)
			} else {
				repl := typo(string(se))
				err := encoder.Flush()
				if err != nil {
					log.Println(err)
				}
				sb.WriteString(repl)
			}
		case xml.EndElement:
			if res, ok := tagsMap[se.Name.Local]; ok {
				if _, ok := parentsMap[res]; ok {
					parent = res
					continue
				}
				if res == "h" {
					res += parent
					parent = ""
				}
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

var replaces = map[*regexp.Regexp]string{
	regexp.MustCompile("\\+-"):            entity("plusmn"),
	regexp.MustCompile("(\\s+)(--?|—|-)"): entity("nbsp") + entity("mdash"),
	regexp.MustCompile("(^)(--?|—|-)"):    entity("mdash"),
	regexp.MustCompile("\\.{2,}"):         entity("hellip"),
}

func typo(s string) string {
	result := s
	for reg, repl := range replaces {
		result = reg.ReplaceAllString(result, repl)
	}

	return result
}
