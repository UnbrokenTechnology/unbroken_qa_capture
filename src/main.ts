import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { Quasar, Notify, Dialog, Loading } from 'quasar'
import iconSet from 'quasar/icon-set/material-icons'
import router from './router'

// Import icon libraries
import '@quasar/extras/material-icons/material-icons.css'

// Import Quasar css
import 'quasar/src/css/index.sass'

// Import app component
import App from './App.vue'

const app = createApp(App)

// Use Pinia for state management
app.use(createPinia())

// Use Vue Router
app.use(router)

// Use Quasar
app.use(Quasar, {
  plugins: { Notify, Dialog, Loading },
  iconSet: iconSet,
})

app.mount('#app')
