package main

type BiliUserInfo struct {
	Name string `json:"uname"`
}

type NavRes struct {
	Code    int64        `json:"code"`
	Message string       `json:"message"`
	Data    BiliUserInfo `json:"data"`
}

type MangaRes struct {
	Code    int64       `json:"code"`
	Message string      `json:"message"`
	Data    MangaDetail `json:"data"`
}

type EpInfo struct {
	Title      string `json:"title"`
	ShortTitle string `json:"short_title"`
	IsInFree   bool   `json:"is_in_free"`
	IsLocked   bool   `json:"is_locked"`
	ID         int64  `json:"id"`
	Order      int64  `json:"ord"`
}

type MangaDetail struct {
	ID     int64    `json:"id"`
	Title  string   `json:"title"`
	EpList []EpInfo `json:"ep_list"`
}

type EpisodeRes struct {
	Code    int64         `json:"code"`
	Message string        `json:"message"`
	Data    EpisodeDetail `json:"data"`
}

type EpisodeDetail struct {
	Path string `json:"path"`
	Host string `json:"host"`
}

type ImageIndex struct {
	Pics []string `json:"pics"`
}

type ImageToken struct {
	URL   string `json:"url"`
	Token string `json:"token"`
}

type ImageTokenRes struct {
	Code    int64        `json:"code"`
	Message string       `json:"message"`
	Data    []ImageToken `json:"data"`
}
