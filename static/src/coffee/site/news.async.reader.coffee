$ ->
  more = $("button#moreNews").click ->
    ov = $("input#offsetValue")
    lv = $("input#limitValue")
    offset = ov.val()
    limit = lv.val()
    $.get('/recent.atom?offset=' + offset + '&limit=' + limit, onRssSuccess)
    ov.attr("value", parseInt(offset) + parseInt(limit))
  more.button()

  month = $('div#accordion > div > ul > li > a')
  month.click (event) ->
    href = event.target.hash
    vars = href.split('&')
    y = 1
    m = 1
    for v in vars
      pair = v.split('=')
      if pair[0] == 'month'
          m = pair[1]
       else
          y = pair[1]
    dlLog = $("body").find("dl#blog")
    dlLog.empty()
    dlLog.append("<dt>Загрузка данных. Пожалуйста подождите ...</dt>")
    $.get('/recent.atom?year=' + y + '&month=' + m, onArchieveRssSuccess)
  month.button()

onRssSuccess = (xml) ->
  items = $("entry", xml)
  dlLog = $("body").find("dl#log")
  loadBlog(items, xml, dlLog)
  lv = $("input#limitValue")
  limit = lv.val()
  if items.length < parseInt(limit)
    $("button#moreNews").remove()

onArchieveRssSuccess  = (xml) ->
  items = $("entry", xml)
  dlLog = $("body").find("dl#blog")
  dlLog.empty()
  loadBlog(items, xml, dlLog)
  limit = 20
  if items.length < limit
    $("ul.pager").remove()
    $("div.pagination").remove()


loadBlog = (items, xml, dlLog) ->
  items.each (item) ->
    dt = $("entry published", xml).get(item)
    title = $("entry title", xml).get(item)
    description = $("entry content", xml).get(item)
    date = if dt.textContent == undefined then dt.text else dt.textContent
    humanReadableDate = $.format.date(date, "dd MMMM yyyy")

    d = if description.textContent == undefined then description.text else description.textContent
    t = if title.textContent == undefined then title.text else title.textContent

    current_uri = window.location.href
    item_uri = $("entry link", xml).get(item).getAttribute("href")
    if current_uri == item_uri
      link = '<span>' + t + '</span>'
    else
      link = "<a href=\"" + $("entry link", xml).get(item).getAttribute("href") + "\">" + t + "</a>"

    dlLog.append("<dt><small><i class=\"icon-calendar\"></i> <span class=\"shortDateFormat\">" + humanReadableDate + "</span></small>&nbsp;" + link + "</dt>")
    dlLog.append("<dd>" + d + "</dd>")