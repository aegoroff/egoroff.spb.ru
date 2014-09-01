window.LOG = ->
  console?.log? arguments...


window.init_common = ->
  init_loading_button()
  init_password_show_button()
  init_time()
  init_announcement()


window.init_loading_button = ->
  $('body').on 'click', '.btn-loading', ->
    $(this).button 'loading'


window.init_password_show_button = ->
  $('body').on 'click', '.btn-password-show', ->
    $target = $($(this).data 'target')
    $target.focus()
    if $(this).hasClass 'active'
      $target.attr 'type', 'password'
    else
      $target.attr 'type', 'text'


window.init_time = ->
  if $('time').length > 0
    recalculate = ->
      $('time[datetime]').each ->
        date = moment.utc $(this).attr 'datetime'
        diff = moment().diff date , 'days'
        if diff > 25
          $(this).text date.local().format 'YYYY-MM-DD'
        else
          $(this).text date.fromNow()
        $(this).attr 'title', date.local().format 'dddd, MMMM Do YYYY, HH:mm:ss Z'
      setTimeout arguments.callee, 1000 * 45
    recalculate()


window.init_announcement = ->
  $('.alert-announcement button.close').click ->
    sessionStorage?.setItem 'closedAnnouncement', $('.alert-announcement').html()

  if sessionStorage?.getItem('closedAnnouncement') != $('.alert-announcement').html()
    $('.alert-announcement').show()


window.clear_notifications = ->
  $('#notifications').empty()


window.show_notification = (message, category='warning') ->
  clear_notifications()
  return if not message

  $('#notifications').append """
      <div class="alert alert-dismissable alert-#{category}">
        <button type="button" class="close" data-dismiss="alert" aria-hidden="true">&times;</button>
        #{message}
      </div>
    """

window.createDate = (v) ->
  mmt = moment.utc v
  mmt.locale(user_lang())
  timez = jstz.determine().name()
  return mmt.tz(timez)

window.formatDateWith = (v, formatSting) ->
  return createDate(v).format formatSting

window.formatElementWith = (elem, formatSting) ->
  if $(elem).is(":input")
    $(elem).val formatDateWith $(elem).val(), formatSting
  else
    $(elem).text formatDateWith $(elem).text(), formatSting