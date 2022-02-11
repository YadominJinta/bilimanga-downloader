package main

import (
	"io/ioutil"
	"net/http"
	"strings"
)

func DoRequest(method, url, body, cookie, contentType string) ([]byte, error) {
	cli := http.Client{}
	req, err := http.NewRequest(method, url, strings.NewReader(body))
	if err != nil {
		return nil, err
	}
	req.Header.Set("cookie", cookie)
	req.Header.Set("user-agent", DefaultUserAgent)
	if len(contentType) > 0 {
		req.Header.Set("content-type", contentType)
	}
	resp, err := cli.Do(req)
	if err != nil {
		return nil, err
	}
	defer resp.Body.Close()
	respBody, err := ioutil.ReadAll(resp.Body)
	if err != nil {
		return nil, err
	}
	return respBody, nil
}
