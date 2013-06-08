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
    tag_q = ''
    if tag != ''
      tag_q = '&tag=' + tag
    date_q = '?'
    if y > 1 and m > 1
      date_q = '?year=' + y + '&month=' + m

    $.get(api_uri + date_q + tag_q, onArchieveRssSuccess)
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
  ov = $("input#offsetValue")
  lv = $("input#limitValue")
  offset = parseInt(ov.val()) or 0
  limit = parseInt(lv.val()) or 20
  LOG('result count: ' + result.count)
  LOG('limit: ' + limit)
  if result.count < limit
    $("ul.pager").remove()
    $("div.pagination").remove()
  else
    $("ul.pager").parent().append('<button class="btn btn-large btn-primary btn-block" id="moreNews" type="button">Больше записей</button>
<input type="hidden" name="offset" id="offsetValue" value="' + offset + '"/>
<input type="hidden" name="limit" id="limitValue" value="' + limit + '"/>
')
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
