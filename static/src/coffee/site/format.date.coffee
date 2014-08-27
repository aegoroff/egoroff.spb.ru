$ ->
  $(document).ready ->
      $(".shortDateFormat").each (idx, elem) ->
          if $(elem).is(":input")
            mmt = moment($(elem).val())
            mmt.locale('ru')
            $(elem).val(mmt.format('LL'))
          else
            mmt = moment($(elem).text())
            mmt.locale('ru')
            $(elem).text(mmt.format('LL'))

      $(".longDateFormat").each (idx, elem) ->
          if $(elem).is(":input")
            mmt = moment($(elem).val())
            mmt.locale('ru')
            $(elem).val(mmt.format('LLL'))
          else
            mmt = moment($(elem).text())
            mmt.locale('ru')
            $(elem).text(mmt.format('LLL'))

