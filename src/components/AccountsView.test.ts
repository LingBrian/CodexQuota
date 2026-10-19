import { mount, flushPromises } from '@vue/test-utils'
import { createPinia, setActivePinia } from 'pinia'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { invoke } from '@tauri-apps/api/core'
import { useQuotaStore } from '../stores/quota'
import AccountsView from './AccountsView.vue'
vi.mock('@tauri-apps/api/core', () => ({ invoke: vi.fn() }))
beforeEach(() => {
  HTMLDialogElement.prototype.showModal = function () { this.open = true }
  HTMLDialogElement.prototype.close = function () { this.open = false }
  setActivePinia(createPinia()); vi.resetAllMocks(); vi.mocked(invoke).mockResolvedValue([])
})
describe('账户操作确认', () => {
  it('自动发现认证文件并预览；导入失败后释放已消费的预览', async () => {
    vi.mocked(invoke).mockImplementation(async command => {
      if(command==='discover_local_auth') return [{path:'C:\\Users\\test\\.codex\\auth.json',suggestedAlias:'local-codex-1'}]
      if(command==='preview_import') return {ticketId:'ticket',alias:'local-codex-1',sizeBytes:10}
      if(command==='account_action') throw new Error('登录或额度接口验证失败')
      return []
    })
    const view=mount(AccountsView)
    await flushPromises()
    expect(invoke).toHaveBeenCalledWith('discover_local_auth')
    await view.findAll('button').find(b=>b.text()==='选择此账号')!.trigger('click')
    await view.findAll('button').find(b=>b.text()==='预览导入')!.trigger('click')
    await flushPromises()
    expect(invoke).toHaveBeenCalledWith('preview_import',{path:'C:\\Users\\test\\.codex\\auth.json',alias:'local-codex-1'})
    await view.findAll('button').find(b=>b.text()==='确认导入…')!.trigger('click')
    await view.find('dialog').findAll('button').find(b=>b.text()==='确认')!.trigger('click')
    await flushPromises()
    expect((view.find('dialog').element as HTMLDialogElement).open).toBe(false)
    expect(view.find('[role="alert"]').text()).toContain('登录或额度接口验证失败')
    expect(view.find('.import-preview').exists()).toBe(false)
  })
  it('切换按钮必须经过确认才调用后端', async () => {
    const store = useQuotaStore()
    store.data.accounts = [{ alias: 'work', current: false, available: true }]
    const view = mount(AccountsView)
    await flushPromises()
    await view.findAll('button').find(button => button.text() === '切换')!.trigger('click')
    expect(invoke).not.toHaveBeenCalledWith('account_action', expect.anything())
    expect(view.find('dialog').text()).toContain('运行中的工作')
    vi.mocked(invoke).mockImplementation(async command => command === 'account_action' ? { source: 'CLI', stale: false, accounts: [] } : [])
    await view.find('dialog').findAll('button').find(button => button.text() === '确认')!.trigger('click')
    await flushPromises()
    expect(invoke).toHaveBeenCalledWith('account_action', { action: { kind: 'switch', alias: 'work' }, confirmed: true })
  })
  it('当前账号的切换和删除按钮被禁用', async () => {
    const store = useQuotaStore()
    store.data.accounts = [{ alias: 'active', current: true, available: true }]
    const view = mount(AccountsView)
    await flushPromises()
    expect(view.findAll('button').find(button => button.text() === '删除')!.attributes('disabled')).toBeDefined()
  })
})
