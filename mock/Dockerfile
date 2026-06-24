FROM node:24@sha256:fdddfb3e688158251943d52eba361de991548f6814007acba4917ae6b512d6be
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
