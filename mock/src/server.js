const express = require('express')
const session = require('express-session')
const fs = require('fs')
const bodyParser = require('body-parser')


const app = express()
const port = 3000
const global = {
  loginCount: 0,
  logoutCount: 0
}

app.use(session({
  name: 'connect.sid',
  secret: 'Unsecure cookies signing key',
  resave: false,
  saveUninitialized: true,
  cookie: {
    maxAge: 10 * 60 * 1000
  }
}))

app.use(bodyParser.urlencoded({ extended: false }))

app.use((req, res, next) => {
  console.info(`${Date.now()}: ${req.method} ${req.url}`)
  next()
})

app.get('/mock/login-count', (req, res) => {
  res.send(`${global.loginCount}`)
})

app.get('/mock/logout-count', (req, res) => {
  res.send(`${global.logoutCount}`)
})

app.post('/web/login', (req, res) => {
  global.loginCount++
  const email = req.body.email
  const password = req.body.password
  if (email === 'registered-user@rika-firenet.com' || password === 'Secret') {
    req.session.user = email
    req.session.stoves = {
      12345: readStoveStatusTemplate('12345'),
      333444: readStoveStatusTemplate('333444'),
    };
    res.body = 'Found. Redirecting to /web/summary'
    res.redirect("/web/summary")
  } else {
    res.body = 'Found. Redirecting to /web/login'
    res.redirect("/web/login")
  }
})

function readStoveStatusTemplate(stoveId) {
  const text = fs.readFileSync('stove-status.json', 'utf8')
    .replaceAll('__stove_id__', stoveId);
  return JSON.parse(text)
}

app.get('/web/logout', (req, res) => {
  global.logoutCount++
  if (req.session.user) {
    req.session.destroy()
  }
  res.body = 'Found. Redirecting to /web/login'
  res.redirect('/web/login')
})

app.get('/web/summary', (req, res) => {
  if (!req.session.user) {
    res.body = 'Found. Redirecting to /web/'
    res.redirect('/web/')
  } else {
    const summaryBody = fs.readFileSync('summary.html', 'utf8')
    res.send(summaryBody)
  }
})

app.get('/api/client/:stoveId/status', (req, res) => {
  const stoveId = req.params.stoveId
  if (!req.session.user) {
    res.body = 'Authorisation required!'
    res.sendStatus(401)
  } else if (req.session.stoves[stoveId]) {
    res.send(req.session.stoves[stoveId]);
  } else {
    res.body = `Stove ${stoveId} is not registered for user ${req.session.user}`
    res.sendStatus(500)
  }
})

app.post('/api/client/:stoveId/controls', (req, res) => {
  const stoveId = req.params.stoveId
  if (!req.session.user) {
    res.body = 'Authorisation required!'
    res.sendStatus(401)
  } else if (req.session.stoves[stoveId]) {
    req.session.stoves[stoveId].controls = {
      ...req.session.stoves[stoveId].controls,
      ...req.body.operatingMode ?{operatingMode: Number.parseInt(req.body.operatingMode)}:{},
      ...req.body.heatingPower?{heatingPower: Number.parseInt(req.body.heatingPower)}:{},
      ...req.body.targetTemperature?{targetTemperature: req.body.targetTemperature}:{},
    }
    res.send('OK');
  } else {
    res.body = `Stove ${stoveId} is not registered for user ${req.session.user}`
    res.sendStatus(500)
  }
})

app.listen(port, () => {
  console.log(`Rika Firenet mock listening on port ${port}`)
})
