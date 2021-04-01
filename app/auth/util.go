package auth

import (
	"crypto/rand"
	"egoroff.spb.ru/app/lib"
	"encoding/base64"
	"encoding/json"
	"github.com/gin-gonic/contrib/sessions"
	"github.com/gin-gonic/gin"
	"log"
	"net/http"
)

func randToken() string {
	b := make([]byte, 32)
	_, err := rand.Read(b)
	if err != nil {
		log.Println(err)
	}
	return base64.StdEncoding.EncodeToString(b)
}

func updateSession(c *gin.Context, action func(s sessions.Session)) {
	session := sessions.Default(c)
	action(session)
	err := session.Save()
	if err != nil {
		log.Println(err)
	}
}

func fetchGithubUserInfo(url string) (*GithubUserInfo, error) {
	resp, err := http.Get(url)
	if err != nil {
		return nil, err
	}
	defer lib.Close(resp.Body)

	var result GithubUserInfo

	if err := json.NewDecoder(resp.Body).Decode(&result); err != nil {
		return nil, err
	}
	return &result, nil
}
