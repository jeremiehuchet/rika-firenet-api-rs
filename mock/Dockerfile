FROM node:22@sha256:a84054eb2d34e7de0ea6df4c21e9706b3ae4805fc5610684b8609877aaa5efb8
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
