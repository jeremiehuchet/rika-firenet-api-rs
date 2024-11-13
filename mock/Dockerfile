FROM node:20@sha256:df14706cfff86aa0d77d8e5b39642d41209953e3ccb74bddc1519efc52094f1a
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
