package db

import (
	"cloud.google.com/go/datastore"
	"context"
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
	k := datastore.NameKey("Auth", name, nil)
	var auth domain.Auth

	r.query(func(c *datastore.Client, ctx context.Context) {
		err := c.Get(ctx, k, &auth)
		if err != nil {
			log.Println(err)
		}
	})

	return &auth
}

func (r *Repository) AuthKeys() []string {
	var keys []*datastore.Key
	r.query(func(c *datastore.Client, ctx context.Context) {
		q := datastore.NewQuery("Auth").KeysOnly()
		var auths []*domain.Auth
		k, err := c.GetAll(ctx, q, &auths)
		if err != nil {
			log.Println(err)
		}
		keys = k
	})

	result := make([]string, len(keys))
	for i, k := range keys {
		result[i] = k.Name
	}
	return result
}

func (r *Repository) Tags() []*domain.TagContainter {
	var containers []*domain.TagContainter

	r.query(func(c *datastore.Client, ctx context.Context) {
		q := datastore.NewQuery("Post").Filter("is_public=", true).Project("tags", "created", "__key__")

		_, err := c.GetAll(ctx, q, &containers)
		if err != nil {
			log.Println(err)
		}
	})

	return containers
}

func (r *Repository) PostKeys() []*datastore.Key {
	var keys []*datastore.Key
	r.query(func(c *datastore.Client, ctx context.Context) {
		q := datastore.NewQuery("Post").KeysOnly()
		var posts []*domain.Post
		k, err := c.GetAll(ctx, q, &posts)
		if err != nil {
			log.Println(err)
		}
		keys = k
	})

	return keys
}

func (r *Repository) Folders() []*domain.Folder {
	var folders []*domain.Folder

	r.query(func(c *datastore.Client, ctx context.Context) {
		q := datastore.NewQuery("Folder").Filter("is_public=", true)
		_, err := c.GetAll(ctx, q, &folders)
		if err != nil {
			log.Println(err)
		}
	})

	return folders
}

func (r *Repository) File(k *datastore.Key) *domain.File {
	var file domain.File
	r.query(func(c *datastore.Client, ctx context.Context) {
		err := c.Get(ctx, k, &file)
		if err != nil {
			log.Println(err)
		}
	})

	return &file
}

func (r *Repository) FileByUid(uid string) *domain.File {
	var files []*domain.File

	r.query(func(c *datastore.Client, ctx context.Context) {
		q := datastore.NewQuery("File").Filter("uid=", uid)
		_, err := c.GetAll(ctx, q, &files)
		if err != nil {
			log.Println(err)
		}
	})

	if len(files) == 0 {
		return nil
	}

	return files[0]
}

func (r *Repository) Blob(uid string) *domain.Blob {
	var blob domain.Blob

	r.query(func(c *datastore.Client, ctx context.Context) {
		k := datastore.NameKey("_blobmigrator_BlobKeyMapping", uid, nil)
		err := c.Get(ctx, k, &blob)
		if err != nil {
			log.Println(err)
		}
	})
	return &blob
}

func (r *Repository) Post(id int64) *domain.Post {
	var post domain.Post
	r.query(func(c *datastore.Client, ctx context.Context) {
		k := datastore.IDKey("Post", id, nil)
		err := c.Get(ctx, k, &post)
		if err != nil {
			log.Println(err)
		}
	})

	return &post
}

func (r *Repository) query(action func(c *datastore.Client, ctx context.Context)) {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	action(c, ctx)
}
