window.LOG = () ->
  console?.log?(arguments...)


window.init_loading_button = () ->
  $('body').on 'click', '.btn-loading', ->
    $(this).button('loading')

window.user_lang = () ->
  language = 'en'
  if navigator.userLanguage
    language = navigator.userLanguage
  else if navigator.language
    language = navigator.language
  return language


window.createDate = (v) ->
  mmt = moment.utc(v)
  mmt.locale(user_lang())
  timez = jstz.determine().name()
  return mmt.tz(timez)

window.formatDateWith = (v, formatSting) ->
  return createDate(v).format(formatSting)

window.formatElementWith = (elem, formatSting) ->
  if $(elem).is(":input")
    $(elem).val(formatDateWith($(elem).val(), formatSting))
  else
    $(elem).text(formatDateWith($(elem).text(), formatSting))