$ ->
  if window.location.pathname != '/'
    return
  window.setInterval( ->
    api_uri = '/api/v2/post.random.json'
    $.get(api_uri, onSuccess)
  , 10000)


onSuccess = (result) ->
  b = $("body")
  r = result.result

  container = b.find("div.hero-unit")
  container.empty()
  container.append('<h1>' + r["title"] + '</h1>')
  container.append('<p>' + r["short_text"] + '</p>')


  base = window.location.origin
  item_uri = base + '/blog/' + r["id"] + '.html'

  container.append('<p><a class="btn btn-primary btn-large" href="' + item_uri + '">Читать далее ...</a></p>')
