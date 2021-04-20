package micropub

import (
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"fmt"
	"time"
)

type store struct {
	repo *db.Repository
}

func newStore(repo *db.Repository) *store {
	return &store{repo: repo}
}

func (m *store) Entry(url string) (data map[string][]interface{}, err error) {
	return map[string][]interface{}{}, nil
}

func (m *store) Create(data map[string][]interface{}) (string, error) {
	title, _ := data["name"][0].(string)
	content, ok := data["content"][0].(map[string]interface{})
	var text string
	if ok {
		html, ok := content["html"]
		if ok {
			text = html.(string)
		} else {
			txt, ok := content["text"]
			if ok {
				text = txt.(string)
			}
		}
	} else {
		content, ok := data["content"][0].(string)
		if ok {
			text = content
		}
	}
	post := domain.Post{
		Model: domain.Model{
			Created: time.Now(),
		},
		IsPublic: false,
		Title:    title,
		Text:     text,
	}
	k, err := m.repo.NewPost(&post)
	return fmt.Sprintf("https://www.egoroff.spb.ru/blog/%d.html", k), err
}

func (m *store) Update(url string, replace, add, delete map[string][]interface{}, deleteAlls []string) error {
	return nil
}

func (m *store) Delete(url string) error {
	return nil
}

func (m *store) Undelete(url string) error {
	return nil
}
