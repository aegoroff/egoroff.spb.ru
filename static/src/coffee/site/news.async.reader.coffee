$ ->
  api_uri = '/api/v2/posts.json'

  month = $('div#accordion > div > ul > li > a')
  month.click (event) ->
    href = event.target.hash
    $('div#accordion > div > ul > li').removeClass('active')
    $(this).parent('li').addClass('active')
    vars = href.split('&')
    y = 1
    m = 1
    for v in vars
      pair = v.split('=')
      if pair[0] == 'month'
          m = parseInt(pair[1], 10)
       else
          y = parseInt(pair[1], 10)
    dlLog = $("body").find("dl#blog")
    dlLog.empty()
    dlLog.append("<dt>Загрузка данных. Пожалуйста подождите ...</dt>")

    mmt = moment(new Date(y, m - 1, 10))
    mmt.locale(user_lang())

    $('ul.breadcrumb > li.active').text("Записи за " + mmt.format('MMMM YYYY'))
    service_call('get', api_uri, { "year" : y, "month" : m }, onArchieveRssSuccess)
  month.button()

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
        title = item["title"]
        description = item["short_text"]

        base = window.location.origin
        current_uri = window.location.href
        item_uri = base + '/blog/' + item["id"] + '.html'
        link = "<a href=\"" + item_uri + "\">" + title + "</a>"
        if current_uri == item_uri
          link = '<span>' + title + '</span>'
        dlLog.append("<dt><small><i class=\"icon-calendar\"></i> <span class=\"shortDateFormat\">" + formatDateWith(date, 'LL') + "</span></small>&nbsp;" + link + "</dt>")
        dlLog.append("<dd>" + description + "</dd>")
