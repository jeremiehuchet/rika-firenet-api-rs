const express = require("express");
const session = require("express-session");
const fs = require("fs");
const bodyParser = require("body-parser");

const app = express();
const port = 3000;
const global = {
  loginCount: 0,
  logoutCount: 0,
};

app.use(
  session({
    name: "connect.sid",
    secret: "Unsecure cookies signing key",
    resave: false,
    saveUninitialized: true,
    cookie: {
      maxAge: 10 * 60 * 1000,
    },
  })
);

app.use(bodyParser.urlencoded({ extended: false }));

app.use((req, res, next) => {
  console.info(`${Date.now()}: ${req.method} ${req.url}`);
  next();
});

app.get("/mock/login-count", (req, res) => {
  res.send(`${global.loginCount}`);
});

app.get("/mock/logout-count", (req, res) => {
  res.send(`${global.logoutCount}`);
});

app.post("/web/login", (req, res) => {
  global.loginCount++;
  const email = req.body.email;
  const password = req.body.password;
  if (email === "registered-user@rika-firenet.com" || password === "Secret") {
    req.session.user = email;
    req.session.stoves = {
      12345: readStoveStatusTemplate("12345"),
      333444: readStoveStatusTemplate("333444"),
    };
    res.body = "Found. Redirecting to /web/summary";
    res.redirect("/web/summary");
  } else {
    res.body = "Found. Redirecting to /web/login";
    res.redirect("/web/login");
  }
});

function readStoveStatusTemplate(stoveId) {
  const text = fs
    .readFileSync("stove-status.json", "utf8")
    .replaceAll("__stove_id__", stoveId);
  return JSON.parse(text);
}

app.get("/web/logout", (req, res) => {
  global.logoutCount++;
  if (req.session.user) {
    req.session.destroy();
  }
  res.body = "Found. Redirecting to /web/login";
  res.redirect("/web/login");
});

app.get("/web/summary", (req, res) => {
  if (!req.session.user) {
    res.body = "Found. Redirecting to /web/";
    res.redirect("/web/");
  } else {
    const summaryBody = fs.readFileSync("summary.html", "utf8");
    res.send(summaryBody);
  }
});

app.get("/api/client/:stoveId/status", (req, res) => {
  const stoveId = req.params.stoveId;
  if (!req.session.user) {
    res.body = "Authorisation required!";
    res.sendStatus(401);
  } else if (req.session.stoves[stoveId]) {
    res.send(req.session.stoves[stoveId]);
  } else {
    res.body = `Stove ${stoveId} is not registered for user ${req.session.user}`;
    res.sendStatus(500);
  }
});

app.post("/api/client/:stoveId/controls", (req, res) => {
  const stoveId = req.params.stoveId;
  if (!req.session.user) {
    res.body = "Authorisation required!";
    res.sendStatus(401);
  } else if (req.session.stoves[stoveId]) {
    req.session.stoves[stoveId].controls = {
      ...req.session.stoves[stoveId].controls,
      ...(req.body.onOff === true || req.body.onOff === false ? { onOff: req.body.onOff } : {}),
      ...(req.body.operatingMode ? { operatingMode: Number.parseInt(req.body.operatingMode) } : {}),
      ...(req.body.heatingPower ? { heatingPower: Number.parseInt(req.body.heatingPower) } : {}),
      ...(req.body.targetTemperature ? { targetTemperature: req.body.targetTemperature } : {}),
      ...(req.body.setBackTemperature ? { setBackTemperature: req.body.setBackTemperature } : {}),
      ...(req.body.heatingTimesActiveForComfort === true || req.body.heatingTimesActiveForComfort === false ? { heatingTimesActiveForComfort: req.body.heatingTimesActiveForComfort } : {}),
      ...(req.body.heatingTimeMon1 ? { heatingTimeMon1: req.body.heatingTimeMon1 } : {}),
      ...(req.body.heatingTimeMon2 ? { heatingTimeMon2: req.body.heatingTimeMon2 } : {}),
      ...(req.body.heatingTimeTue1 ? { heatingTimeTue1: req.body.heatingTimeTue1 } : {}),
      ...(req.body.heatingTimeTue2 ? { heatingTimeTue2: req.body.heatingTimeTue2 } : {}),
      ...(req.body.heatingTimeWed1 ? { heatingTimeWed1: req.body.heatingTimeWed1 } : {}),
      ...(req.body.heatingTimeWed2 ? { heatingTimeWed2: req.body.heatingTimeWed2 } : {}),
      ...(req.body.heatingTimeThu1 ? { heatingTimeThu1: req.body.heatingTimeThu1 } : {}),
      ...(req.body.heatingTimeThu2 ? { heatingTimeThu2: req.body.heatingTimeThu2 } : {}),
      ...(req.body.heatingTimeFri1 ? { heatingTimeFri1: req.body.heatingTimeFri1 } : {}),
      ...(req.body.heatingTimeFri2 ? { heatingTimeFri2: req.body.heatingTimeFri2 } : {}),
      ...(req.body.heatingTimeSat1 ? { heatingTimeSat1: req.body.heatingTimeSat1 } : {}),
      ...(req.body.heatingTimeSat2 ? { heatingTimeSat2: req.body.heatingTimeSat2 } : {}),
      ...(req.body.heatingTimeSun1 ? { heatingTimeSun1: req.body.heatingTimeSun1 } : {}),
      ...(req.body.heatingTimeSun2 ? { heatingTimeSun2: req.body.heatingTimeSun2 } : {}),
    };
    console.info(`Updated controls for ${stoveId}:`, JSON.stringify(req.session.stoves[stoveId].controls, null, 2))
    res.send("OK");
  } else {
    res.body = `Stove ${stoveId} is not registered for user ${req.session.user}`;
    res.sendStatus(500);
  }
});

app.listen(port, () => {
  console.log(`Rika Firenet mock listening on port ${port}`);
});
