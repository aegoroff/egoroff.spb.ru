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

    mmt = moment(new Date(y, m - 1, 10))
    mmt.locale(user_lang())

    $('ul.breadcrumb > li.active').text("Записи за " + mmt.format('MMMM YYYY'))
    window.LoadBlog({ "year" : y, "month" : m })
    $("ul.pager").remove()
    $("div.pagination").remove()

  month.button()
