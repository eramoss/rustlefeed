document.addEventListener('DOMContentLoaded', function () {
  const contentContainer = document.getElementById('feed-item');
  const likeBtn = document.getElementById('like');
  const dislikeBtn = document.getElementById('dislike');

  async function fetchNextContent(is_liked) {
    const response = await fetch('/next', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ liked: is_liked }),
    });
    if (response.ok) {
      window.scrollTo({ top: 0, behavior: 'smooth' });
      const content = await response.text();
      setTimeout(() => {
        contentContainer.innerHTML = content;
      }, 200);
    } else {
      console.error('Failed to fetch next content');
    }
  }

  likeBtn.addEventListener('click', function () {
    fetchNextContent(true);
  });
  dislikeBtn.addEventListener('click', function () {
    fetchNextContent(false);
  });
  fetchNextContent(null);
});

const feedList = document.getElementById('feed-list');
const newFeedInput = document.getElementById('new-feed-input');
const addFeedButton = document.getElementById('add-feed-button');

function createFeedItem(feed) {
  const li = document.createElement('li');
  li.textContent = feed.title;

  const removeButton = document.createElement('button');
  removeButton.className = 'remove-button';
  const iconSpan = document.createElement('span');
  iconSpan.className = 'icon';
  iconSpan.innerText = '×';
  removeButton.appendChild(iconSpan);

  removeButton.addEventListener('click', function () {
    fetch('/delete-feed', {
      method: 'POST',
      headers: {
        'Content-Type': 'application/json',
      },
      body: JSON.stringify({ url: feed.url }),
    })
      .then((response) => {
        if (response.ok) {
          feedList.removeChild(li);
        } else {
          console.error('Failed to delete feed:', response.statusText);
        }
      })
      .catch((error) => console.error('Error deleting feed:', error));
  });

  li.appendChild(removeButton);
  return li;
}

function addNewFeed(url) {
  fetch('/add-feed', {
    method: 'POST',
    headers: {
      'Content-Type': 'application/json',
    },
    body: JSON.stringify({ url }),
  })
    .then((response) => {
      if (response.ok) {
        return response.json();
      } else {
        throw new Error('Failed to add feed:', response.statusText);
      }
    })
    .then((_) => {
      listFeeds();
    })
    .catch((error) => console.error(error));
}

function listFeeds() {
  fetch('/feeds')
    .then((response) => {
      if (response.ok) {
        return response.json();
      } else {
        throw new Error('Failed to fetch feeds:', response.statusText);
      }
    })
    .then((feeds) => {
      while (feedList.firstChild) {
        feedList.removeChild(feedList.firstChild);
      }
      feeds.forEach((feed) => {
        const feedItem = createFeedItem(feed);
        feedList.appendChild(feedItem);
      });
    })
    .catch((error) => console.error(error));

}

addFeedButton.addEventListener('click', function () {
  const url = newFeedInput.value.trim();
  if (url !== '') {
    addNewFeed(url);
    newFeedInput.value = '';
  }
});

listFeeds();
