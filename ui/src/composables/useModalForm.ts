import { nextTick, onMounted, onUnmounted, ref } from 'vue'
import { useNotify } from '@/composables/useNotify'
import { closeModalById } from '@/util'

export interface ModalFormOptions {
  modalId: string
  onHidden?: () => void
}

// Tabs are switched through Bootstrap's data API, so an invalid field on a hidden
// tab pane becomes visible before it receives focus.
function revealInvalidField(form: HTMLFormElement): void {
  const invalid = form.querySelector<HTMLElement>(':invalid')
  if (!invalid) {
    return
  }
  const pane = invalid.closest<HTMLElement>('.tab-pane')
  if (pane && !pane.classList.contains('active')) {
    form.querySelector<HTMLElement>(`[data-bs-target="#${pane.id}"]`)?.click()
  }
  invalid.focus()
}

export function useModalForm(options: ModalFormOptions) {
  const notify = useNotify()
  const form = ref<HTMLFormElement | null>(null)
  const validated = ref(false)
  const busy = ref(false)

  const preventHideWhileBusy = (e: Event): void => {
    if (busy.value) {
      e.preventDefault()
    }
  }

  const onHidden = (): void => {
    validated.value = false
    options.onHidden?.()
  }

  onMounted(() => {
    const modal = document.getElementById(options.modalId)
    modal?.addEventListener('hide.bs.modal', preventHideWhileBusy)
    modal?.addEventListener('hidden.bs.modal', onHidden)
  })

  onUnmounted(() => {
    const modal = document.getElementById(options.modalId)
    modal?.removeEventListener('hide.bs.modal', preventHideWhileBusy)
    modal?.removeEventListener('hidden.bs.modal', onHidden)
  })

  /** Validates the form, runs `action` with the modal locked and closes the modal on success. */
  const submit = async (action: () => Promise<void>, errorText: string): Promise<boolean> => {
    validated.value = true
    const el = form.value
    if (!el?.checkValidity()) {
      if (el) {
        revealInvalidField(el)
      }
      notify.error('Заполните обязательные поля')
      return false
    }
    busy.value = true
    try {
      await action()
    } catch (error) {
      notify.error(errorText, error)
      return false
    } finally {
      busy.value = false
    }
    // The dismiss button stays disabled until Vue re-renders after `busy` is cleared.
    await nextTick()
    closeModalById(options.modalId)
    return true
  }

  return { form, validated, busy, submit }
}
