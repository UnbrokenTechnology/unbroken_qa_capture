import { createApp } from 'vue'
import { createPinia } from 'pinia'
import { Quasar } from 'quasar'

// Import icon libraries
import '@quasar/extras/material-icons/material-icons.css'

// Import Quasar css
import 'quasar/src/css/index.sass'

// Import app component
import App from './App.vue'

const app = createApp(App)

// Use Pinia for state management
app.use(createPinia())

// Use Quasar
app.use(Quasar, {
  plugins: {}, // import Quasar plugins and add here
})

app.mount('#app')
