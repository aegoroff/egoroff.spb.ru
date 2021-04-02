package auth

import (
	"egoroff.spb.ru/app/auth/oauth"
	"egoroff.spb.ru/app/db"
	"egoroff.spb.ru/app/domain"
	"encoding/json"
	"io/ioutil"
	"log"
)

func CreateOrUpdateProviders(path string) {
	file, err := ioutil.ReadFile(path)
	if err != nil {
		log.Println(err)
		return
	}
	var providers []*oauth.AuthProvider
	err = json.Unmarshal(file, &providers)
	if err != nil {
		log.Println(err)
		return
	}
	repo := db.NewRepository()
	for _, provider := range providers {
		a := repo.Auth(provider.Name)
		if a != nil {
			continue
		}

		a = &domain.Auth{
			ClientID:     provider.Credentials.ClientID,
			ClientSecret: provider.Credentials.ClientSecret,
			RedirectURL:  provider.RedirectURL,
			Scopes:       provider.Scopes,
		}
		err = repo.NewAuth(a, provider.Name)
		if err != nil {
			log.Println(err)
		}
	}
}
