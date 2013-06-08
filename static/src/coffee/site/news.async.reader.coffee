$ ->
  api_uri = '/api/v2/posts.json'
  more = $("button#moreNews").click ->
    ov = $("input#offsetValue")
    lv = $("input#limitValue")
    offset = ov.val()
    limit = lv.val()
    $.get(api_uri + '?offset=' + offset + '&limit=' + limit, onRssSuccess)
    ov.attr("value", parseInt(offset) + parseInt(limit))
  more.button()

  month = $('a.q')
  month.click (event) ->
    href = event.target.hash
    vars = href.split('&')
    y = 1
    m = 1
    tag = ''
    for v in vars
      pair = v.split('=')
      switch pair[0]
        when "month" then m = pair[1]
        when "year" then y = pair[1]
        when "#year" then y = pair[1]
        when "#tag" then tag = pair[1]
        when "tag" then tag = pair[1]
        else y = pair[1]

    dlLog = $("body").find("dl#blog")
    dlLog.empty()
    dlLog.append("<dt>Загрузка данных. Пожалуйста подождите ...</dt>")
    if tag != ''
      $.get(api_uri + '?tag=' + tag, onArchieveRssSuccess)
    else
      $.get(api_uri + '?year=' + y + '&month=' + m, onArchieveRssSuccess)
  month.button()

onRssSuccess = (result) ->
  dlLog = $("body").find("dl#log")
  loadBlog(result.result, dlLog)
  lv = $("input#limitValue")
  limit = lv.val()
  if result.count < parseInt(limit)
    $("button#moreNews").remove()

onArchieveRssSuccess  = (result) ->
  dlLog = $("body").find("dl#blog")
  dlLog.empty()
  loadBlog(result.result, dlLog)
  limit = 20
  if result.count < limit
    $("ul.pager").remove()
    $("div.pagination").remove()


loadBlog = (items, dlLog) ->
  for item in items
      date = item["created"]
      date = date.substring(0, date.length - 4)
      title = item["title"]
      description = item["short_text"]
      humanReadableDate = $.format.date(date, "dd MMMM yyyy")

      base = window.location.origin
      current_uri = window.location.href
      item_uri = base + '/news/' + item["id"] + '.html'
      link = "<a href=\"" + item_uri + "\">" + title + "</a>"
      if current_uri == item_uri
        link = '<span>' + title + '</span>'
      dlLog.append("<dt><small><i class=\"icon-calendar\"></i> <span class=\"shortDateFormat\">" + humanReadableDate + "</span></small>&nbsp;" + link + "</dt>")
      dlLog.append("<dd>" + description + "</dd>")
