package txt

import (
	"strings"

	"golang.org/x/net/html"
)

func ParseHtml(s string) string {
	z := html.NewTokenizer(strings.NewReader(s))
	sb := strings.Builder{}
	skip := ""
	for {
		if z.Next() == html.ErrorToken {
			// Returning io.EOF indicates success.
			break
		}

		tok := z.Token()

		switch tok.Type {
		case html.TextToken:
			if skipElement(skip) {
				sb.WriteString(html.EscapeString(tok.Data))
			} else {
				sb.WriteString(typo(tok.Data))
			}
		case html.SelfClosingTagToken:
			sb.WriteRune('<')
			sb.WriteString(tok.Data)
			sb.WriteString("/>")
		case html.StartTagToken:
			sb.WriteRune('<')

			if skipElement(tok.Data) {
				if skip == "" {
					skip = tok.Data
				}
			}

			sb.WriteString(tok.Data)
			for _, attribute := range tok.Attr {
				sb.WriteRune(' ')
				sb.WriteString(attribute.Key)
				sb.WriteString("=\"")
				sb.WriteString(attribute.Val)
				sb.WriteRune('"')
			}
			// HACK
			if tok.Data == "table" && len(tok.Attr) == 0 {
				sb.WriteString(" class=\"table table-condensed\"")
			}

			sb.WriteRune('>')
		case html.EndTagToken:
			sb.WriteString("</")
			sb.WriteString(tok.Data)
			sb.WriteRune('>')
			if skip == tok.Data {
				skip = ""
			}
		}
	}
	return sb.String()
}
