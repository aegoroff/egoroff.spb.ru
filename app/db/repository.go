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
	k := datastore.NameKey("Config", "master", nil)
	var config domain.Config

	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		return c.Get(ctx, k, &config)
	})
	if err != nil {
		return nil, err
	}

	return &config, nil
}

func (r *Repository) UserByFederatedId(sub string) (*domain.User, error) {
	var users []*domain.User

	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		q := datastore.NewQuery("User").Filter("federated_id=", sub)
		_, err := c.GetAll(ctx, q, &users)
		return err
	})
	if err != nil || len(users) == 0 {
		log.Println(err)
		return nil, err
	}

	return users[0], nil
}

func (r *Repository) NewUser(user *domain.User) error {
	k := datastore.IncompleteKey("User", nil)
	return r.UpdateUser(user, k)
}

func (r *Repository) UpdateUser(user *domain.User, k *datastore.Key) error {
	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		_, err := c.Put(ctx, k, user)
		return err
	})
	if err != nil {
		log.Println(err)
	}

	return err
}

func (r *Repository) NewAuth(user *domain.Auth, name string) error {
	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		k := datastore.NameKey("Auth", name, nil)
		_, err := c.Put(ctx, k, user)
		return err
	})
	if err != nil {
		log.Println(err)
		return err
	}

	return nil
}

func (r *Repository) Auth(name string) *domain.Auth {
	k := datastore.NameKey("Auth", name, nil)
	var auth domain.Auth

	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		return c.Get(ctx, k, &auth)
	})
	if err != nil {
		log.Println(err)
	}

	return &auth
}

func (r *Repository) AuthKeys() []string {
	var keys []*datastore.Key
	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		q := datastore.NewQuery("Auth").KeysOnly()
		var auths []*domain.Auth
		k, err := c.GetAll(ctx, q, &auths)
		if err != nil {
			return err
		}
		keys = k
		return nil
	})
	if err != nil {
		log.Println(err)
		return nil
	}

	result := make([]string, len(keys))
	for i, k := range keys {
		result[i] = k.Name
	}
	return result
}

func (r *Repository) Tags() []*domain.TagContainter {
	var containers []*domain.TagContainter

	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		q := datastore.NewQuery("Post").Filter("is_public=", true).Project("tags", "created", "__key__")

		_, err := c.GetAll(ctx, q, &containers)
		return err
	})
	if err != nil {
		log.Println(err)
	}

	return containers
}

func (r *Repository) PostKeys() []*datastore.Key {
	var keys []*datastore.Key
	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		q := datastore.NewQuery("Post").KeysOnly()
		var posts []*domain.Post
		k, err := c.GetAll(ctx, q, &posts)
		if err != nil {
			return err
		}
		keys = k
		return nil
	})
	if err != nil {
		log.Println(err)
	}

	return keys
}

func (r *Repository) Folders() []*domain.Folder {
	var folders []*domain.Folder

	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		q := datastore.NewQuery("Folder").Filter("is_public=", true)
		_, err := c.GetAll(ctx, q, &folders)
		return err
	})
	if err != nil {
		log.Println(err)
	}

	return folders
}

func (r *Repository) File(k *datastore.Key) *domain.File {
	var file domain.File
	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		return c.Get(ctx, k, &file)
	})
	if err != nil {
		log.Println(err)
	}

	return &file
}

func (r *Repository) FileByUid(uid string) *domain.File {
	var files []*domain.File

	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		q := datastore.NewQuery("File").Filter("uid=", uid)
		_, err := c.GetAll(ctx, q, &files)
		return err
	})
	if err != nil || len(files) == 0 {
		log.Println(err)
		return nil
	}

	return files[0]
}

func (r *Repository) Blob(uid string) *domain.Blob {
	var blob domain.Blob

	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		k := datastore.NameKey("_blobmigrator_BlobKeyMapping", uid, nil)
		return c.Get(ctx, k, &blob)
	})
	if err != nil {
		log.Println(err)
		return nil
	}
	return &blob
}

func (r *Repository) Post(id int64) *domain.Post {
	var post domain.Post
	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		k := datastore.IDKey("Post", id, nil)
		return c.Get(ctx, k, &post)
	})
	if err != nil {
		return nil
	}

	return &post
}

func (r *Repository) NewPost(post *domain.Post) (int64, error) {
	k := datastore.IncompleteKey("Post", nil)
	err := r.query(func(c *datastore.Client, ctx context.Context) error {
		newK, err := c.Put(ctx, k, post)
		k = newK
		return err
	})
	if err != nil {
		log.Println(err)
		return 0, err
	}

	return k.ID, nil
}

func (r *Repository) query(action func(c *datastore.Client, ctx context.Context) error) error {
	c, err := r.conn.Connect()
	if err != nil {
		log.Println(err)
		return err
	}
	defer lib.Close(c)

	ctx, cancel := r.conn.newContext()
	defer cancel()

	return action(c, ctx)
}
