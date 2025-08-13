FROM node:22@sha256:5cc527147c43e671934c3530e9f66863162db5f57ce1ed0f227a3bb57b93cfab
WORKDIR /mock
COPY package*.json ./
RUN npm ci --omit=dev
COPY src .
EXPOSE 3000
CMD [ "node", "server.js" ]
