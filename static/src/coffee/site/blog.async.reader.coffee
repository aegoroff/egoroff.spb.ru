$ ->
  api_uri = '/api/v2/posts.json'

  removePager = ->
    $("ul.pager").remove()
    $("div.pagination").remove()

  removeSelection = ->
    $('div.tags > ul > li > a').removeClass('btn')
    $('div#accordion > div > ul > li').removeClass('active')

  setBreadcrumbsText = (txt) ->
    $('ul.breadcrumb > li.active').text(txt)

  month = $('div#accordion > div > ul > li > a')
  month.click (event) ->
    href = event.target.hash
    removeSelection()
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

    $("dl#blogcontainer").remove()

    mmt = moment(new Date(y, m - 1, 10))
    mmt.locale(user_lang())

    setBreadcrumbsText("Записи за " + mmt.format('MMMM YYYY'))
    window.LoadBlog({ "year" : y, "month" : m })
    removePager()
  month.button()

  tag = $('div.tags > ul > li > a')
  tag.click (event) ->
    removeSelection()
    $(this).addClass('btn')
    href = event.target.hash
    $("dl#blogcontainer").remove()
    t = href.split('=')[1]
    setBreadcrumbsText('Все посты по метке: ' + t)
    window.LoadBlog({ "tag" : t })
    removePager()
  tag.button()

