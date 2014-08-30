window.init_admin_config = () ->
  init_loading_button()
  LOG('init adminka')
  $('nav#adminka > ul.nav > li > a').click (event) ->
    $('nav#adminka > ul.nav > li.active').removeClass('active')
    $(this).parent('li').addClass('active')
    return
  return
