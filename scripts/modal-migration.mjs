import fs from 'node:fs'
const file='src/components/AccountsView.vue'
const source=fs.readFileSync(file,'utf8').replace('<dialog :open="!!pending"','<dialog ref="dialog" @cancel="cancel"').replace('<button :disabled="busy" @click="pending = undefined">','<button autofocus :disabled="busy" @click="pending = undefined">')
fs.writeFileSync(file,source)
