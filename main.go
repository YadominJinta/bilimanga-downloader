package main

import (
	"archive/zip"
	"bytes"
	"encoding/json"
	"errors"
	"flag"
	"fmt"
	"io"
	"log"
	"net/http"
	"net/url"
	"os"
	"strconv"
	"strings"
	"sync"
)

func DownloadPic(filename, url, cookie string) error {
	resp, err := DoRequest(http.MethodGet, url, "", cookie, "")
	if err != nil {
		return err
	}
	return os.WriteFile(filename, resp, 0644)
}

func DownloadEpisode(mangaInfo *MangaDetail, epInfo *EpInfo, cookie string, episodePath string) error {
	const EpisodeIndexURL = BiliMangaBaseURL + "/twirp/comic.v1.Comic/GetImageIndex?device=pc&platform=web"
	const ImageTokenURL = BiliMangaBaseURL + "/twirp/comic.v1.Comic/ImageToken?device=pc&platform=web"
	// Get episode's image index
	resp, err := DoRequest(http.MethodPost, EpisodeIndexURL,
		fmt.Sprintf("{\"ep_id\":%d}", epInfo.ID), cookie, JsonContent)
	if err != nil {
		return err
	}
	epRes := EpisodeRes{}
	err = json.Unmarshal(resp, &epRes)
	if err != nil {
		return err
	}
	// Get encoded data
	resp, err = DoRequest(http.MethodGet, epRes.Data.Host+epRes.Data.Path, "", cookie, "")
	if err != nil {
		return err
	}
	// Decode data
	body := DecodeIndex(mangaInfo.ID, epInfo.ID, resp[9:])
	reader, err := zip.NewReader(bytes.NewReader(body), int64(len(body)))
	if err != nil {
		return err
	}
	// Extract from zip
	index := ImageIndex{}
	f, err := reader.File[0].Open()
	defer f.Close()
	if err != nil {
		return err
	}
	b, err := io.ReadAll(f)
	if err != nil {
		return err
	}
	err = json.Unmarshal(b, &index)
	if err != nil {
		return err
	}
	// Get full URLs
	urls, err := json.Marshal(index.Pics)
	if err != nil {
		return err
	}
	s, err := json.Marshal(struct {
		Urls string `json:"urls"`
	}{
		Urls: string(urls),
	})
	checkErr(err)
	resp, err = DoRequest(http.MethodPost, ImageTokenURL, string(s), cookie, JsonContent)
	if err != nil {
		return err
	}
	imageTokens := ImageTokenRes{}
	err = json.Unmarshal(resp, &imageTokens)
	if err != nil {
		return err
	}
	// Download images
	wg := sync.WaitGroup{}
	for i, v := range imageTokens.Data {
		wg.Add(1)
		go func(i int, v ImageToken) {
			defer wg.Done()
			err := DownloadPic(fmt.Sprintf("%s/%d.jpg", episodePath, i+1),
				fmt.Sprintf("%s?token=%s", v.URL, v.Token), cookie)
			if err != nil {
				log.Println(err)
			}
		}(i, v)
	}
	wg.Wait()
	return nil
}

func GetMe(cookie string) (string, error) {
	const NavURL = BiliApiBaseURL + "/x/web-interface/nav"
	resp, err := DoRequest(http.MethodGet, NavURL, "", cookie, JsonContent)
	if err != nil {
		return "", err
	}
	body := NavRes{}
	err = json.Unmarshal(resp, &body)
	if err != nil {
		return "", err
	}
	if body.Code != 0 {
		return "", errors.New(body.Message)
	}
	return body.Data.Name, nil
}

func GetMangaDetail(mangaID int64, cookie string) (*MangaDetail, error) {
	const MangaDetailURL = BiliMangaBaseURL + "/twirp/comic.v1.Comic/ComicDetail?device=pc&platform=web"
	resp, err := DoRequest(http.MethodPost, MangaDetailURL,
		fmt.Sprintf("{\"comic_id\":%d}", mangaID), cookie, JsonContent)
	if err != nil {
		return nil, err
	}
	body := MangaRes{}
	err = json.Unmarshal(resp, &body)
	if err != nil {
		return nil, err
	}
	if body.Code != 0 {
		return nil, errors.New(body.Message)
	}
	return &body.Data, nil
}

func GetMangaID(mangaURL string) (int64, error) {
	u, err := url.Parse(mangaURL)
	if err != nil {
		return 0, err
	}
	splits := strings.Split(u.Path, "/")
	mcID := splits[len(splits)-1]
	mcID = strings.Replace(mcID, "mc", "", -1)
	id, err := strconv.Atoi(mcID)
	if err != nil {
		return 0, err
	}
	return int64(id), nil
}

func main() {
	var (
		Cookie     = ""
		CookieFile = ""
		MangaURL   = ""
	)

	flag.StringVar(&Cookie, "cookie", "", "your cookie on bilibili manga")
	flag.StringVar(&CookieFile, "cookie-file", "", "your cookie file")
	flag.StringVar(&MangaURL, "url", "", "manga url to download")
	flag.Parse()

	if len(Cookie) == 0 && len(CookieFile) == 0 {
		println("You must set one of cookie or cookie-file")
		flag.Usage()
		os.Exit(1)
	}

	if len(MangaURL) == 0 {
		println("You must set manga url")
		flag.Usage()
		os.Exit(1)
	}

	if len(Cookie) == 0 {
		cookieFile, err := os.Open(CookieFile)
		checkErr(err)
		cookieContent, err := io.ReadAll(cookieFile)
		checkErr(err)
		Cookie = string(cookieContent)
	}

	mangaID, err := GetMangaID(MangaURL)
	checkErr(err)
	myName, err := GetMe(Cookie)
	checkErr(err)
	mangaInfo, err := GetMangaDetail(mangaID, Cookie)
	checkErr(err)

	log.Printf("Hello, %s\n", myName)
	log.Printf("You're downloading %s\n", mangaInfo.Title)

	if _, err := os.Stat(mangaInfo.Title); os.IsNotExist(err) {
		err = os.Mkdir(mangaInfo.Title, 0755)
		checkErr(err)
	}

	for _, ep := range mangaInfo.EpList {
		if !ep.IsLocked || ep.IsInFree {
			BasePath := fmt.Sprintf("./%s/%d. %s %s", mangaInfo.Title, ep.Order, ep.ShortTitle, ep.Title)
			if st, err := os.Stat(BasePath); os.IsNotExist(err) {
				if err = os.Mkdir(BasePath, 0755); err != nil {
					checkErr(err)
				}
			} else if !st.IsDir() {
				err = os.Remove(BasePath)
				checkErr(err)
				err = os.Mkdir(BasePath, 0755)
				checkErr(err)
			} else {
				log.Printf("%s: %s has been downloaded, skip", ep.ShortTitle, ep.Title)
				continue
			}
			log.Printf("Started to download %s: %s", ep.ShortTitle, ep.Title)
			_ = DownloadEpisode(mangaInfo, &ep, Cookie, BasePath)
			continue
		}
		log.Printf("%s: %s is locked", ep.ShortTitle, ep.Title)
	}
}

func checkErr(err error) {
	if err != nil {
		log.Panicln(err)
	}
}
