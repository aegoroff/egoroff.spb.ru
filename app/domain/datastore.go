package domain

import (
	"cloud.google.com/go/datastore"
	"time"
)

type Model struct {
	Key         *datastore.Key `datastore:"__key__"`
	Created     time.Time      `datastore:"created"`
	Modified    time.Time      `datastore:"modified"`
	Version     int            `datastore:"version"`
	CreatedAgo  string         `datastore:"created_ago"`
	CreatedUtc  string         `datastore:"created_utc"`
	ModifiedAgo string         `datastore:"modified_ago"`
	ModifiedUtc string         `datastore:"modified_utc"`
}

type Post struct {
	Model
	Id        int64    `json:"id" datastore:"-"`
	IsPublic  bool     `datastore:"is_public"`
	Tags      []string `datastore:"tags"`
	Title     string   `datastore:"title"`
	ShortText string   `datastore:"short_text"`
	Text      string   `datastore:"text,noindex"`
}

type SmallPost struct {
	Key       *datastore.Key `datastore:"__key__"`
	Created   time.Time      `datastore:"created"`
	Id        int64          `json:"id"`
	Title     string         `datastore:"title"`
	ShortText string         `datastore:"short_text"`
}

type TagContainter struct {
	Key     *datastore.Key `datastore:"__key__"`
	Tags    []string       `datastore:"tags"`
	Created time.Time      `datastore:"created"`
}

type Config struct {
	Model
	AnalyticsId           string `datastore:"analytics_id"`
	AnnouncementType      string `datastore:"announcement_type"`
	AnnouncementHtml      string `datastore:"announcement_html"`
	AnonymousRecaptcha    bool   `datastore:"anonymous_recaptcha"`
	BrandName             string `datastore:"brand_name"`
	CheckUniqueEmail      bool   `datastore:"check_unique_email"`
	FacebookAppId         string `datastore:"facebook_app_id"`
	FacebookAppSecret     string `datastore:"facebook_app_secret"`
	FeedbackEmail         string `datastore:"feedback_email"`
	FlaskSecretKey        string `datastore:"flask_secret_key"`
	GoogleSiteId          string `datastore:"google_site_id"`
	NotifyOnNewUser       bool   `datastore:"notify_on_new_user"`
	RecaptchaPrivateKey   string `datastore:"recaptcha_private_key"`
	RecaptchaPublicKey    string `datastore:"recaptcha_public_key"`
	SearchApiKey          string `datastore:"search_api_key"`
	TwitterConsumerKey    string `datastore:"twitter_consumer_key"`
	TwitterConsumerSecret string `datastore:"twitter_consumer_secret"`
	VerifyEmail           bool   `datastore:"verify_email"`
}

type User struct {
	Model
	Active      bool     `datastore:"active"`
	Admin       bool     `datastore:"admin"`
	AuthIds     []string `datastore:"auth_ids"`
	AvatarUrl   string   `datastore:"avatar_url"`
	Email       string   `datastore:"email"`
	FacebookId  string   `datastore:"facebook_id"`
	FederatedId string   `datastore:"federated_id"`
	Name        string   `datastore:"name"`
	Token       string   `datastore:"token"`
	TwitterId   string   `datastore:"twitter_id"`
	Username    string   `datastore:"username"`
	Verified    bool     `datastore:"verified"`
	Provider    string   `datastore:"provider"`
}

func (u *User) String() string {
	if u.Name != "" {
		return u.Name
	} else if u.Username != "" {
		return u.Username
	} else {
		return u.Email
	}
}

type File struct {
	Model
	BlobKey           string         `datastore:"blob_key"`
	ContentType       string         `datastore:"content_type"`
	Description       string         `datastore:"description"`
	Ext               string         `datastore:"ext"`
	Filename          string         `datastore:"filename"`
	HumanReadableSize string         `datastore:"human_readable_size"`
	IsImage           bool           `datastore:"is_image"`
	IsPublic          bool           `datastore:"is_public"`
	Size              int64          `datastore:"size"`
	Title             string         `datastore:"title"`
	TitleFilename     string         `datastore:"title_filename"`
	Uid               string         `datastore:"uid"`
	Url               string         `datastore:"url"`
	Owner             *datastore.Key `datastore:"owner"`
}

type Blob struct {
	Key         *datastore.Key `datastore:"__key__"`
	GcsFileName string         `datastore:"gcs_filename"`
	NewBlobKey  string         `datastore:"new_blob_key"`
	OldBlobKey  string         `datastore:"old_blob_key"`
}

type Folder struct {
	Model
	IsPublic bool             `datastore:"is_public"`
	Title    string           `datastore:"title"`
	FileKeys []*datastore.Key `datastore:"files"`
	Files    []*File
}

type Auth struct {
	Key          *datastore.Key `datastore:"__key__"`
	ClientID     string         `datastore:"clientid"`
	ClientSecret string         `datastore:"secret"`
	RedirectURL  string         `datastore:"redirect_url"`
	Scopes       []string       `datastore:"scopes"`
}
