package db

import (
	"cloud.google.com/go/datastore"
	"egoroff.spb.ru/app/domain"
	"egoroff.spb.ru/app/lib"
	"log"
	"time"
)

type Repository struct {
	conn *Connection
}

func NewRepository() *Repository {
	return &Repository{
		conn: NewConnection(5 * time.Second),
	}
}

func (r *Repository) Config() (*domain.Config, error) {
	c, err := r.conn.Connect()
	if err != nil {
		return nil, err
	}
	defer lib.Close(c)
	k := datastore.NameKey("Config", "master", nil)
	var config domain.Config

	ctx, cancel := r.conn.newContext()
	defer cancel()

	err = c.Get(ctx, k, &config)
	if err != nil {
		return nil, err
	}

	return &config, nil
}

func (r *Repository) UserByFederatedId(sub string) (*domain.User, error) {
	c, err := r.conn.Connect()
	if err != nil {
		return nil, err
	}
	defer lib.Close(c)
	var users []*domain.User

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("User").Filter("federated_id=", sub)
	_, err = c.GetAll(ctx, q, &users)
	if err != nil || len(users) == 0 {
		return nil, err
	}

	return users[0], nil
}

func (r *Repository) NewUser(user *domain.User) error {
	k := datastore.IncompleteKey("User", nil)
	return r.UpdateUser(user, k)
}

func (r *Repository) UpdateUser(user *domain.User, k *datastore.Key) error {
	c, err := r.conn.Connect()
	if err != nil {
		return err
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	_, err = c.Put(ctx, k, user)
	if err != nil {
		return nil
	}

	return nil
}

func (r *Repository) NewAuth(user *domain.Auth, name string) error {
	c, err := r.conn.Connect()
	if err != nil {
		return err
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	k := datastore.NameKey("Auth", name, nil)
	_, err = c.Put(ctx, k, user)
	if err != nil {
		return nil
	}

	return nil
}

func (r *Repository) Auth(name string) *domain.Auth {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	k := datastore.NameKey("Auth", name, nil)
	var auth domain.Auth
	err = c.Get(ctx, k, &auth)
	if err != nil {
		log.Println(err)
		return nil
	}
	return &auth
}

func (r *Repository) AuthKeys() []string {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("Auth").KeysOnly()
	var auths []*domain.Auth
	keys, err := c.GetAll(ctx, q, &auths)
	if err != nil {
		log.Println(err)
	}
	result := make([]string, len(keys))
	for i, k := range keys {
		result[i] = k.Name
	}
	return result
}

func (r *Repository) Tags() []*domain.TagContainter {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("Post").Filter("is_public=", true).Project("tags", "created", "__key__")
	var containters []*domain.TagContainter
	_, err = c.GetAll(ctx, q, &containters)
	if err != nil {
		log.Println(err)
	}
	return containters
}

func (r *Repository) PostKeys() []*datastore.Key {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("Post").KeysOnly()
	var posts []*domain.Post
	keys, err := c.GetAll(ctx, q, &posts)
	if err != nil {
		log.Println(err)
	}
	return keys
}

func (r *Repository) Folders() []*domain.Folder {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	q := datastore.NewQuery("Folder").Filter("is_public=", true)
	var folders []*domain.Folder
	_, err = c.GetAll(ctx, q, &folders)
	if err != nil {
		log.Println(err)
	}
	return folders
}

func (r *Repository) File(k *datastore.Key) *domain.File {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	var file domain.File
	err = c.Get(ctx, k, &file)
	if err != nil {
		log.Println(err)
	}
	return &file
}

func (r *Repository) FileByUid(uid string) *domain.File {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	var files []*domain.File
	q := datastore.NewQuery("File").Filter("uid=", uid)
	_, err = c.GetAll(ctx, q, &files)
	if err != nil || len(files) == 0 {
		log.Println(err)
		return nil
	}
	return files[0]
}

func (r *Repository) Blob(uid string) *domain.Blob {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	k := datastore.NameKey("_blobmigrator_BlobKeyMapping", uid, nil)
	var blob domain.Blob
	err = c.Get(ctx, k, &blob)
	if err != nil {
		log.Println(err)
		return nil
	}
	return &blob
}

func (r *Repository) Post(id int64) *domain.Post {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return nil
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	k := datastore.IDKey("Post", id, nil)
	var post domain.Post
	err = c.Get(ctx, k, &post)
	if err != nil {
		log.Println(err)
	}
	return &post
}
