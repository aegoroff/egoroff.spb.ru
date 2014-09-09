$ ->
  api_uri = '/api/v2/posts.json'

  removePager = ->
    $("ul.pager").remove()
    $("ul.pagination").remove()

  removeSelection = ->
    $('div.tags > ul > li > a').removeClass('btn btn-default')
    $('div#accordion > div > a').removeClass('active')

  month = $('div#accordion > div > a')
  month.click (event) ->
    href = event.target.hash
    removeSelection()
    $(this).addClass('active')
    vars = href.split('&')
    y = 1
    m = 1
    for v in vars
      pair = v.split('=')
      if pair[0] == 'month'
          m = parseInt(pair[1], 10)
       else
          y = parseInt(pair[1], 10)

    $("dl#blogcontainer").remove()

    mmt = moment(new Date(y, m - 1, 10))
    mmt.locale(user_lang())

    txt = "записи за " + mmt.format('MMMM YYYY');
    window.LoadBlog({ "year" : y, "month" : m }, txt)
    removePager()
  month.button()

  tag = $('div.tags > ul > li > a')
  tag.click (event) ->
    removeSelection()
    $(this).addClass('btn btn-default')
    href = event.target.hash
    $("dl#blogcontainer").remove()
    t = href.split('=')[1]
    txt = 'все посты по метке: ' + t
    window.LoadBlog({ "tag" : t }, txt)
    removePager()
  tag.button()

