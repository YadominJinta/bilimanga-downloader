# B站漫画下载器

因为 python 版本单线程太慢了，所以用 go 写了一个。

## 使用

```shell
$ go install github.com/YadominJinta/bilimanga-downloader@latest

$ bilimanga-downloader
Usage of bilimanga-downloader
  -cookie string
        your cookie on bilibili manga
  -url string
        manga url to download
```

## 特性

- 多线程
- 自动跳过已下载和锁定的章节

## References

- [bilibili_manga_downloader](https://github.com/xuruoyu/bilibili_manga_downloader)