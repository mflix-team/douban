#!/usr/bin/env python
# -*- encoding: utf-8 -*-
'''
@File: douban.py
@Description: 
@Time: 2021/08/08 16:22:48
@Author: iptoday
@Email: wangdong1221@outlook.com
@Version: 1.0.0
'''

import sys
import time
import requests
import timeit


headers = {
    "Accept": "*/*",
    "User-Agent": "Mozilla/5.0 (Linux; Android 16; Pixel 10) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/153.0.0.0 Mobile Safari/537.36",
    "Referer": "https://m.douban.com/",
}


def main():
    start = timeit.default_timer()
    args = sys.argv
    if len(args) > 1:
        for i in range(0, len(args)):
            if i > 0:
                if len(args) > 2 and i > 1:
                    print("PLEASE WAIT, GET NEXT MOVE DATA IN 5 SECIONS LATER.")
                    time.sleep(5)
                get_html(args[i])
    else:
        print("END. Not Found DouBan Ids.")
    end = timeit.default_timer()
    print("END %ss" % round((end-start), 2))


# 从豆瓣抓取条目信息，返回 dict
def fetch_subject(subject_id, timeout=15):
    # 移动端 API（电影/剧集通用，无需登录）
    api_url = "https://m.douban.com/rexxar/api/v2/movie/%s" % subject_id
    response = requests.get(api_url, headers=headers, timeout=timeout)
    if response.status_code != 200:
        raise ValueError("mobile api status %s" % response.status_code)
    raw = response.json()

    resource_type = "tv" if raw.get("is_tv") else "movie"
    rating_num = (raw.get("rating") or {}).get("value") or 0
    title = raw.get("title", "")
    year = raw.get("year", "") or ""
    cover = (raw.get("pic") or {}).get("large") or ""
    directors = ", ".join(d.get("name", "") for d in (raw.get("directors") or []))
    summary = (raw.get("intro") or "").replace("\n", "").replace(" ", "")

    actors = [a.get("name", "") for a in (raw.get("actors") or [])]
    writers = []

    # 编剧不在主接口，从 credits 演职员表提取（roles 包含"编剧"）
    credits_url = "https://m.douban.com/rexxar/api/v2/movie/%s/credits" % subject_id
    try:
        credits = requests.get(credits_url, headers=headers, timeout=timeout).json()
        for item in (credits.get("items") or []):
            if "编剧" in (item.get("roles") or []):
                writers.append(item.get("name", ""))
            if not actors and ("演员" in (item.get("roles") or []) or "饰" in (item.get("character") or "")):
                actors.append(item.get("name", ""))
    except Exception:
        pass

    actors = list(dict.fromkeys(actors))
    writers = ", ".join(dict.fromkeys(writers))
    actors_str = ", ".join(actors)

    genres = raw.get("genres") or []
    releases = raw.get("pubdate") or []
    runtime = " ".join(raw.get("durations") or []) if raw.get("durations") else ""
    translation = " ".join(raw.get("aka") or []) if raw.get("aka") else ""

    if isinstance(genres, str):
        genres = [genres]
    if isinstance(releases, str):
        releases = [releases]

    return {
        "id": subject_id,
        "resource_type": resource_type,
        "rating_num": rating_num,
        "title": title,
        "year": year,
        "cover": cover,
        "genres": genres,
        "director": directors,
        "writers": writers,
        "actors": actors_str,
        "summary": summary,
        "releases": releases,
        "runtime": runtime,
        "translation": translation,
    }


# 获取网页内容并解析
def get_html(id):
    url = "https://movie.douban.com/subject/"+id
    print("START GET %s DATA." % url)
    try:
        data = fetch_subject(id)
        print("RATING NUM: %s" % data["rating_num"])
        print("TITLE: %s" % data["title"])
        print("YEAR: %s" % data["year"])
        print("COVER: %s" % data["cover"])
        print("DIRECTORS: %s" % data["director"])
        print("WRITERS: %s" % data["writers"])
        print("ACTORS: %s" % data["actors"])
        print("SUMMARY: %s" % data["summary"])
        print("RELEASES: %s" % data["releases"])
        print("RUNTIME: %s" % data["runtime"])
        print("TRANSLATION: %s" % data["translation"])
        print("DATA: %s" % data)
    except Exception as e:
        print("UNKNOW ERROR: %s" % e)


if __name__ == "__main__":
    main()
