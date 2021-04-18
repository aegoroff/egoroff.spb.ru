package micropub

import (
	"egoroff.spb.ru/app"
	"github.com/gin-gonic/gin"
)

type endpoint struct {
}

func NewEndpoint() app.Router {
	return &endpoint{}
}

func (s *endpoint) Route(r *gin.Engine) {
	mock := mockDB{}
	geth := getHandler(&mock, "/media", []SyndicateTo{})
	posth := postHandler(&mock)
	mpub := r.Group("/micropub")
	{
		mpub.GET("/", geth)
		mpub.POST("/", posth)
	}
}

type mockDB struct {
}

func (m *mockDB) Entry(url string) (data map[string][]interface{}, err error) {
	return map[string][]interface{}{}, nil
}

func (m *mockDB) Create(data map[string][]interface{}) (string, error) {
	return "", nil
}

func (m *mockDB) Update(url string, replace, add, delete map[string][]interface{}, deleteAlls []string) error {
	return nil
}

func (m *mockDB) Delete(url string) error {
	return nil
}

func (m *mockDB) Undelete(url string) error {
	return nil
}
